use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{
    attributes::ItemAttrs,
    error::Error,
    util::{parse_struct_fields, Field},
};

pub struct DeriveProtobufModel {
    fields: Vec<Field<ItemAttrs>>,
    ident: syn::Ident,
    vis: syn::Visibility,
}

impl DeriveProtobufModel {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = parse_struct_fields::<ItemAttrs>(input.data)?;

        let ident = input.ident;
        let vis = input.vis;

        Ok(DeriveProtobufModel { fields, ident, vis })
    }

    pub fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_protobuf_schema = self.expand_impl_protobuf_schema()?;

        Ok(TokenStream::from_iter([expanded_impl_protobuf_schema]))
    }

    fn expand_impl_protobuf_schema(&self) -> syn::Result<TokenStream> {
        let Self {
            fields, ident, vis, ..
        } = self;

        let protobuf_schema_ident = format_ident!("{}ProtobufSchema", ident);
        let message_name = ident.to_string();

        let fields = fields
            .iter()
            .map(|field| {
                let name = field.field.ident.as_ref().unwrap().to_string();
                let ty = if let Some(db_type) = &field.attrs.proto_type {
                    db_type.value()
                } else if let Some(db_type) = Self::rust_to_proto_type(&field.field.ty) {
                    db_type.to_string()
                } else {
                    return Err(syn::Error::new(
                        field.field.ty.span(),
                        "type is not suppoerted",
                    ));
                };
                let required = !Self::is_type_option(&field.field.ty);

                Ok(quote!(
                    awto_schema::protobuf::ProtobufField {
                        name: #name.to_string(),
                        ty: #ty.to_string(),
                        required: #required,
                    }
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let fields_len = fields.len();

        Ok(quote!(
            #[derive(Clone, Copy, Default)]
            #vis struct #protobuf_schema_ident;

            impl awto_schema::protobuf::IntoProtobufSchema for #ident {
                type Schema = #protobuf_schema_ident;
            }

            impl awto_schema::protobuf::ProtobufSchema for #protobuf_schema_ident {
                fn message_name(&self) -> &'static str {
                    #message_name
                }

                fn fields(&self) -> ::std::vec::Vec<awto_schema::protobuf::ProtobufField> {
                    let mut fields = Vec::with_capacity(#fields_len + awto_schema::protobuf::DEFAULT_PROTOBUF_FIELDS.len());
                    fields.extend(awto_schema::protobuf::DEFAULT_PROTOBUF_FIELDS.clone());
                    fields.extend([
                        #( #fields, )*
                    ]);
                    fields
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

    fn rust_to_proto_type(ty: &syn::Type) -> Option<&'static str> {
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
            "i16" | "i32" => "int32",
            "u16" | "u32" => "uint32",
            "i64" => "int64",
            "u64" => "uint64",
            "f32" => "float",
            "f64" => "double",

            // Character types
            "String" | "&str" => "string",

            // Binary data types
            "Vec<u8>" | "&u8" => "bytes",

            // Date/Time types
            "chrono::NaiveDateTime" | "NaiveDateTime" => "google.protobuf.Timestamp",
            "chrono::DateTime" | "DateTime" => "google.protobuf.Timestamp",
            "chrono::NaiveDate" | "NaiveDate" => "google.protobuf.Timestamp",
            "chrono::NaiveTime" | "NaiveTime" => "google.protobuf.Timestamp",

            // Boolean type
            "bool" => "bool",

            // Uuid type
            "uuid::Uuid" | "Uuid" => "string",

            _ => return None,
        };

        Some(db_type)
    }
}

pub fn expand_derive_protobuf_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveProtobufModel::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveProtobufModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
