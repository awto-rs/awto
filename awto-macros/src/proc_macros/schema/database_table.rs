use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::{
    attributes::ItemAttrs,
    error::Error,
    util::{parse_fields, Field},
};

pub struct DatabaseTableModel {
    fields: Vec<Field<ItemAttrs>>,
    ident: syn::Ident,
    is_sub_model: bool,
}

impl DatabaseTableModel {
    pub fn new(item: syn::ItemStruct, is_sub_model: bool) -> Result<Self, Error> {
        let punctuated_fields = match item.fields {
            syn::Fields::Named(named) => named.named,
            _ => return Err(Error::FieldsNotNamed),
        };

        let fields = parse_fields::<ItemAttrs>(punctuated_fields)?;

        let ident = item.ident;

        Ok(DatabaseTableModel {
            fields,
            ident,
            is_sub_model,
        })
    }

    pub fn expand(self) -> syn::Result<TokenStream> {
        let expanded_database_table = self.expand_database_table()?;

        Ok(expanded_database_table)
    }
}

impl DatabaseTableModel {
    fn expand_database_table(&self) -> syn::Result<TokenStream> {
        let Self { fields, ident, .. } = self;

        let table_name = ident.to_string().to_snake_case();

        if !self.is_sub_model {
            macro_rules! check_field_exists {
                ($field: literal, $ty: literal) => {
                    if !fields
                        .iter()
                        .any(|field| field.field.ident.as_ref().unwrap() == $field)
                    {
                        return Err(syn::Error::new(
                            ident.span(),
                            concat!(
                                "database models must have an `",
                                $field,
                                ": ",
                                $ty,
                                "` column"
                            ),
                        ));
                    }
                };
            }

            check_field_exists!("id", "Uuid");
            check_field_exists!("created_at", "DateTime<FixedOffset>");
            check_field_exists!("updated_at", "DateTime<FixedOffset>");
        }

        let columns = fields
            .iter()
            .map(|field| {
                let name = field.field.ident.as_ref().unwrap().to_string();

                let field_str = field.field.ty.to_token_stream().to_string().replace(' ', "");
                if name == "id" && field_str != "uuid::Uuid" && field_str != "Uuid" {
                    return Err(syn::Error::new(field.field.ty.span(), "`id` must be of type `Uuid`"));
                }
                if name == "created_at"
                    && field_str != "chrono::DateTime<chrono::FixedOffset>"
                    && field_str != "chrono::DateTime<FixedOffset>"
                    && field_str != "DateTime<chrono::FixedOffset>"
                    && field_str != "DateTime<FixedOffset>"
                {
                    return Err(syn::Error::new(field.field.ty.span(), "`created_at` must be of type `DateTime<FixedOffset>`"));
                }
                if name == "updated_at"
                    && field_str != "chrono::DateTime<chrono::FixedOffset>"
                    && field_str != "chrono::DateTime<FixedOffset>"
                    && field_str != "DateTime<chrono::FixedOffset>"
                    && field_str != "DateTime<FixedOffset>"
                {
                    return Err(syn::Error::new(field.field.ty.span(), "`updated_at` must be of type `DateTime<FixedOffset>`"));
                }

                let mut ty = if let Some(db_type) = &field.attrs.db_type {
                    if let Ok(db_type) = db_type.value().parse::<TokenStream>() {
                        quote!(awto::database::DatabaseType::#db_type)
                    } else {
                        return Err(syn::Error::new(db_type.span(), "invalid db_type"));
                    }
                } else if let Some(db_type) = Self::rust_to_db_type(&field.field.ty) {
                    db_type
                } else {
                    return Err(syn::Error::new(
                        field.field.ty.span(),
                        "type is not suppoerted",
                    ));
                };
                let ty_string = ty.to_string();
                let db_type_is_text = ty_string.ends_with("::Text") || ty_string.ends_with(":: Text");
                if let Some(max_len) = &field.attrs.max_len {
                    if !db_type_is_text {
                        return Err(syn::Error::new(
                            max_len.span(),
                            "max_len can only be used on varchar & char types",
                        ));
                    }
                    ty = quote!(#ty(Some(#max_len)));
                } else if db_type_is_text {
                    ty = quote!(#ty(None));
                }

                let nullable = Self::is_type_option(&field.field.ty);
                if nullable && name == "id" {
                    return Err(syn::Error::new(field.field.ty.span(), "`id` cannot be an Option"));
                }
                if nullable && name == "created_at" {
                    return Err(syn::Error::new(field.field.ty.span(), "`created_at` cannot be an Option"));
                }
                if nullable && name == "updated_at" {
                    return Err(syn::Error::new(field.field.ty.span(), "`updated_at` cannot be an Option"));
                }

                let verify_id_created_at_updated_at_custom_default = || {
                    if name == "id" {
                        return Err(syn::Error::new(field.field.ty.span(), "`id` cannot have a custom default"));
                    }
                    if name == "created_at" {
                        return Err(syn::Error::new(field.field.ty.span(), "`created_at` cannot have a custom default"));
                    }
                    if name == "updated_at" {
                        return Err(syn::Error::new(field.field.ty.span(), "`updated_at` cannot have a custom default"));
                    }
                    Ok(())
                };

                let mut default = if let Some(default_raw) = &field.attrs.default_raw {
                    verify_id_created_at_updated_at_custom_default()?;

                    quote!(Some(awto::database::DatabaseDefault::Raw(#default_raw.to_string())))
                } else if let Some(default) = &field.attrs.default {
                    verify_id_created_at_updated_at_custom_default()?;

                    if let Some(db_default) = Self::lit_to_db_default(default) {
                        quote!(Some(#db_default))
                    } else {
                        return Err(syn::Error::new(
                            default.span(),
                            "default not supported: use a primitive type only",
                        ));
                    }
                } else {
                    quote!(None)
                };
                if name == "id" {
                    default = quote!(Some(awto::database::DatabaseDefault::Raw("uuid_generate_v4()".to_string())))
                } else if name == "created_at" || name == "updated_at" {
                    default = quote!(Some(awto::database::DatabaseDefault::Raw("NOW()".to_string())))
                }

                let unique = field.attrs.unique.is_some();
                if unique && name == "id" {
                    return Err(syn::Error::new(field.field.ty.span(), "`id` cannot be marked as unique"));
                }
                if unique && name == "created_at" {
                    return Err(syn::Error::new(field.field.ty.span(), "`created_at` cannot be marked as unique"));
                }
                if unique && name == "updated_at" {
                    return Err(syn::Error::new(field.field.ty.span(), "`updated_at` cannot be marked as unique"));
                }

                let references = if let Some(references) = &field.attrs.references {
                    if name == "id" {
                        return Err(syn::Error::new(field.field.ty.span(), "`id` cannot reference another table"));
                    }
                    if name == "created_at" {
                        return Err(syn::Error::new(field.field.ty.span(), "`created_at` cannot reference another table"));
                    }
                    if name == "updated_at" {
                        return Err(syn::Error::new(field.field.ty.span(), "`updated_at` cannot reference another table"));
                    }

                    let references_table = &references.0;
                    let references_table_string = references.0.to_string();
                    let references_column = references.1.value();

                    quote!({
                        if !<#references_table as awto::database::IntoDatabaseMessage>::database_message()
                            .columns
                            .iter()
                            .any(|column| column.name == #references_column)
                        {
                            panic!(concat!(
                                "[error] ",
                                file!(),
                                ": column '",
                                #references_column,
                                "' does not exist on table ",
                                #references_table_string
                            ))
                        }

                        Some((
                            <#references_table as awto::database::IntoDatabaseMessage>::database_message().name,
                            #references_column.to_string(),
                        ))
                    })
                } else {
                    quote!(None)
                };

                let primary_key = name == "id";

                Ok(quote!(
                    awto::database::DatabaseColumn {
                        name: #name.to_string(),
                        ty: #ty,
                        nullable: #nullable,
                        default: #default,
                        unique: #unique,
                        constraint: None,
                        primary_key: #primary_key,
                        references: #references,
                    }
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(quote!(
            awto::database::DatabaseTable {
                name: #table_name.to_string(),
                columns: vec![ #( #columns, )* ],
            }
        ))
    }

    fn is_type_option(ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Path(path) => path
                .path
                .segments
                .first()
                .map(|segment| segment.ident == "Option")
                .unwrap_or(false),
            _ => false,
        }
    }

    fn lit_to_db_default(lit: &syn::Lit) -> Option<TokenStream> {
        let db_default = match lit {
            syn::Lit::Bool(b) => quote!(awto::database::DatabaseDefault::Bool(#b)),
            syn::Lit::Float(f) => quote!(awto::database::DatabaseDefault::Float(#f)),
            syn::Lit::Int(i) => quote!(awto::database::DatabaseDefault::Int(#i)),
            syn::Lit::Str(s) => {
                quote!(awto::database::DatabaseDefault::String(#s.to_string()))
            }
            _ => return None,
        };
        Some(db_default)
    }

    fn rust_to_db_type(ty: &syn::Type) -> Option<TokenStream> {
        let ty_string = match ty {
            syn::Type::Reference(reference) => {
                let mut reference = reference.clone();
                reference.lifetime = None;
                quote!(reference.elem.as_ref()).to_string()
            }
            other => quote!(#other).to_string(),
        }
        .replace(' ', "");
        let ty_str = if ty_string.starts_with("Option<") {
            &ty_string[7..(ty_string.len() - 1)]
        } else {
            ty_string.as_str()
        };

        let db_type = match ty_str {
            // Numeric types
            "i16" => quote!(SmallInt),
            "i32" => quote!(Integer),
            "i64" => quote!(BigInt),
            "f32" => quote!(Float),
            "f64" => quote!(Double),

            // Character types
            "String" => quote!(Text),

            // Binary data types
            "Vec<u8>" => quote!(Binary),

            // Date/Time types
            "chrono::NaiveDateTime" | "NaiveDateTime" => quote!(Timestamp),
            "chrono::DateTime<chrono::FixedOffset>"
            | "chrono::DateTime<FixedOffset>"
            | "DateTime<chrono::FixedOffset>"
            | "DateTime<FixedOffset>" => quote!(Timestamptz),
            "chrono::NaiveDate" | "NaiveDate" => quote!(Date),
            "chrono::NaiveTime" | "NaiveTime" => quote!(Time),

            // Boolean type
            "bool" => quote!(Bool),

            // Uuid type
            "uuid::Uuid" | "Uuid" => quote!(Uuid),

            _ => return None,
        };

        Some(quote!(awto::database::DatabaseType::#db_type))
    }
}
