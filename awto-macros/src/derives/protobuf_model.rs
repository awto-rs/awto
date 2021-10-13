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
                let ty = if let Some(proto_type) = &field.attrs.proto_type {
                    if let Ok(proto_type) = proto_type.value().parse::<TokenStream>() {
                        quote!(awto_schema::database::DatabaseType::#proto_type)
                    } else {
                        return Err(syn::Error::new(proto_type.span(), "invalid proto_type"));
                    }
                } else if let Some(proto_type) = Self::rust_to_proto_type(&field.field.ty) {
                    proto_type
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
                        ty: #ty,
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

    fn rust_to_proto_type(ty: &syn::Type) -> Option<TokenStream> {
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

        Self::rust_str_to_proto_type(ty_str)
            .map(|protobuf_type| quote!(awto_schema::protobuf::ProtobufType::#protobuf_type))
    }

    fn rust_str_to_proto_type(ty_str: &str) -> Option<TokenStream> {
        let protobuf_type = match ty_str {
            "f64" => quote!(Double),
            "f32" => quote!(Float),
            "i32" => quote!(Int32),
            "i64" => quote!(Int64),
            "u32" => quote!(Uint32),
            "u64" => quote!(Uint64),
            "bool" => quote!(Bool),
            "String" | "&str" => quote!(String),
            "Vec<u8>" | "&u8" => quote!(Bytes),
            "chrono::NaiveDateTime" | "NaiveDateTime" | "chrono::DateTime" | "DateTime" => {
                quote!(Timestamp)
            }
            "uuid::Uuid" | "Uuid" => quote!(String),
            _ => {
                if ty_str.starts_with("Vec<") {
                    Self::rust_str_to_proto_type(&ty_str[4..(ty_str.len() - 1)]).map(|inner_ty| {
                        quote!(Repeated(::std::boxed::Box::new(awto_schema::protobuf::ProtobufType::#inner_ty)))
                    })?
                } else {
                    return None;
                }
            }
        };

        Some(protobuf_type)
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
