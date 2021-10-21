use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

use crate::{
    attributes::ItemAttrs,
    error::Error,
    util::{parse_fields, Field},
};

pub struct ProtobufMessageModel {
    fields: Vec<Field<ItemAttrs>>,
    ident: syn::Ident,
}

impl ProtobufMessageModel {
    pub fn new(item: syn::ItemStruct) -> Result<Self, Error> {
        let punctuated_fields = match item.fields {
            syn::Fields::Named(named) => named.named,
            _ => return Err(Error::FieldsNotNamed),
        };

        let fields = parse_fields::<ItemAttrs>(punctuated_fields)?;

        let ident = item.ident;

        Ok(ProtobufMessageModel { fields, ident })
    }

    pub fn expand(self) -> syn::Result<TokenStream> {
        let expanded_protobuf_table = self.expand_protobuf_message()?;

        Ok(expanded_protobuf_table)
    }
}

impl ProtobufMessageModel {
    fn expand_protobuf_message(&self) -> syn::Result<TokenStream> {
        let Self { fields, ident } = self;

        let name = ident.to_string();

        let fields = fields
            .iter()
            .map(|field| {
                let name = field.field.ident.as_ref().unwrap().to_string();
                let ty = if let Some(proto_type) = &field.attrs.proto_type {
                    if let Ok(proto_type) = proto_type.value().parse::<TokenStream>() {
                        quote!(awto::protobuf::ProtobufType::#proto_type)
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
                    awto::protobuf::ProtobufField {
                        name: #name.to_string(),
                        ty: #ty,
                        required: #required,
                    }
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(quote!(
            awto::protobuf::ProtobufMessage {
                name: #name.to_string(),
                fields: vec![ #( #fields, )* ],
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
            .map(|protobuf_type| quote!(awto::protobuf::ProtobufType::#protobuf_type))
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
            "chrono::NaiveDateTime"
            | "NaiveDateTime"
            | "chrono::DateTime<chrono::FixedOffset>"
            | "chrono::DateTime<FixedOffset>"
            | "DateTime<chrono::FixedOffset>"
            | "DateTime<FixedOffset>" => {
                quote!(Timestamp)
            }
            "uuid::Uuid" | "Uuid" => quote!(String),
            _ => {
                if ty_str.starts_with("Vec<") {
                    Self::rust_str_to_proto_type(&ty_str[4..(ty_str.len() - 1)]).map(|inner_ty| {
                        quote!(Repeated(::std::boxed::Box::new(awto::protobuf::ProtobufType::#inner_ty)))
                    })?
                } else {
                    let ty_parsed = format_ident!("{}", ty_str);
                    quote!(Custom(<#ty_parsed as awto::protobuf::IntoProtobufMessage>::protobuf_message()))
                }
            }
        };

        Some(protobuf_type)
    }
}
