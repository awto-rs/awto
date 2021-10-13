use std::iter::FromIterator;

use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{
    attributes::ItemAttrs,
    error::Error,
    util::{parse_struct_fields, Field},
};

pub struct DeriveDatabaseModel {
    fields: Vec<Field<ItemAttrs>>,
    ident: syn::Ident,
    vis: syn::Visibility,
}

impl DeriveDatabaseModel {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = parse_struct_fields::<ItemAttrs>(input.data)?;

        let ident = input.ident;
        let vis = input.vis;

        Ok(DeriveDatabaseModel { fields, ident, vis })
    }

    pub fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_database_schema = self.expand_impl_database_schema()?;

        Ok(TokenStream::from_iter([expanded_impl_database_schema]))
    }

    fn expand_impl_database_schema(&self) -> syn::Result<TokenStream> {
        let Self {
            fields, ident, vis, ..
        } = self;

        let database_schema_ident = format_ident!("{}DatabaseSchema", ident);
        let table_name = ident.to_string().to_snake_case();

        let columns = fields
            .iter()
            .map(|field| {
                let name = field.field.ident.as_ref().unwrap().to_string();
                let mut ty = if let Some(db_type) = &field.attrs.db_type {
                    if let Ok(db_type) = db_type.value().parse::<TokenStream>() {
                        quote!(awto_schema::database::DatabaseType::#db_type)
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
                let db_type_is_text = ty.to_string().ends_with(":: Text");
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
                let default = if let Some(default_raw) = &field.attrs.default_raw {
                    quote!(Some(awto_schema::database::DatabaseDefault::Raw(#default_raw.to_string())))
                } else if let Some(default) = &field.attrs.default {
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
                let unique = field.attrs.unique.is_some();
                let references = if let Some(references) = &field.attrs.references {
                    let references_table = references.0.value();
                    let references_column = references.1.value();

                    quote!(Some((#references_table.to_string(), #references_column.to_string())))
                } else {
                    quote!(None)
                };

                Ok(quote!(
                    awto_schema::database::DatabaseColumn {
                        name: #name.to_string(),
                        ty: #ty,
                        // ty: awto_schema::database::DatabaseType::BigInt,
                        nullable: #nullable,
                        default: #default,
                        unique: #unique,
                        constraint: None,
                        primary_key: false,
                        references: #references,
                    }
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let columns_len = columns.len();

        Ok(quote!(
            #[derive(Clone, Copy, Default)]
            #vis struct #database_schema_ident;

            impl awto_schema::database::IntoDatabaseSchema for #ident {
                type Schema = #database_schema_ident;
            }

            impl awto_schema::database::DatabaseSchema for #database_schema_ident {
                fn table_name(&self) -> &'static str {
                    #table_name
                }

                fn columns(&self) -> ::std::vec::Vec<awto_schema::database::DatabaseColumn> {
                    let mut cols = Vec::with_capacity(#columns_len + awto_schema::database::DEFAULT_DATABASE_COLUMNS.len());
                    cols.extend(awto_schema::database::DEFAULT_DATABASE_COLUMNS.clone());
                    cols.extend([
                        #( #columns, )*
                    ]);
                    cols
                }
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
            syn::Lit::Bool(b) => quote!(awto_schema::database::DatabaseDefault::Bool(#b)),
            syn::Lit::Float(f) => quote!(awto_schema::database::DatabaseDefault::Float(#f)),
            syn::Lit::Int(i) => quote!(awto_schema::database::DatabaseDefault::Int(#i)),
            syn::Lit::Str(s) => {
                quote!(awto_schema::database::DatabaseDefault::String(#s.to_string()))
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
            "i16" | "u16" => quote!(SmallInt),
            "i32" | "u32" => quote!(Integer),
            "i64" | "u64" => quote!(BigInt),
            "f32" => quote!(Float),
            "f64" => quote!(Double),

            // Character types
            "String" | "&str" => quote!(Text),

            // Binary data types
            "Vec<u8>" | "&u8" => quote!(Binary),

            // Date/Time types
            "chrono::NaiveDateTime" | "NaiveDateTime" => quote!(Timestamp),
            "chrono::DateTime" | "DateTime" => quote!(Timestamptz),
            "chrono::NaiveDate" | "NaiveDate" => quote!(Date),
            "chrono::NaiveTime" | "NaiveTime" => quote!(Time),

            // Boolean type
            "bool" => quote!(Bool),

            // Uuid type
            "uuid::Uuid" | "Uuid" => quote!(Uuid),

            _ => return None,
        };

        Some(quote!(awto_schema::database::DatabaseType::#db_type))
    }
}

pub fn expand_derive_database_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveDatabaseModel::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveDatabaseModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
