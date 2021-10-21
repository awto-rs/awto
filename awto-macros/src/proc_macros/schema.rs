use std::{fmt, iter::FromIterator};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::{
    error::Error,
    proc_macros::schema::{
        database_table::DatabaseTableModel, protobuf_message::ProtobufMessageModel,
    },
    util::ProcMacro,
};

mod database_table;
mod protobuf_message;

pub struct Structs(pub Vec<syn::ItemStruct>);

impl syn::parse::Parse for Structs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut structs: Vec<syn::ItemStruct> = Vec::new();
        while !input.is_empty() {
            structs.push(input.parse()?);
        }

        Ok(Structs(structs))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Role {
    DatabaseTable,
    DatabaseSubTable(syn::Ident),
    ProtobufMessage,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RoleFromStrError;

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseTable => write!(f, "database_table"),
            Self::DatabaseSubTable(_) => write!(f, "database_sub_table"),
            Self::ProtobufMessage => write!(f, "protobuf_message"),
        }
    }
}

struct Item {
    roles: Vec<Role>,
    item: syn::ItemStruct,
}

pub struct Schema {
    items: Vec<Item>,
}

impl Schema {
    fn parse_input(input: Structs) -> syn::Result<Vec<Item>> {
        input
            .0
            .into_iter()
            .map(|item| {
                let roles: Vec<_> = item
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        attr.parse_meta()
                            .map(|meta| match &meta {
                                syn::Meta::Path(path) => {
                                    path.get_ident().and_then(|ident| {
                                        if ident == "database_table" {
                                            Some(Ok(Role::DatabaseTable))
                                        } else if ident == "protobuf_message" {
                                            Some(Ok(Role::ProtobufMessage))
                                        } else {
                                            None
                                        }
                                    })
                                },
                                syn::Meta::List(list) => {
                                    list.path.get_ident().and_then(|ident|
                                        if ident == "database_sub_table" {
                                            Some(attr.parse_args::<syn::Ident>().map(Role::DatabaseSubTable))
                                        } else {
                                            None
                                        }
                                    )
                                }
                                _ => None,
                            })
                            .ok()
                            .flatten()
                    })
                    .collect::<Result<_, _>>()?;
                if roles.is_empty() {
                    return Err(
                        syn::Error::new(
                            item.ident.span(),
                            "struct must be marked with a role attribute\n\navailable attributes are #[database_table], #[protobuf_message], #[database_sub_table(Parent)]",
                        )
                    );
                }

                // Ensure all structs are public
                if !matches!(item.vis, syn::Visibility::Public(_)) {
                    return Err(syn::Error::new(item.ident.span(), "struct must be public"));
                }

                Ok(Item { roles, item })
            })
            .collect()
    }

    fn parse_models(&self) -> syn::Result<TokenStream> {
        let models: Vec<_> = self
            .items
            .iter()
            .map(|item| {
                let item_ident = item.item.ident.to_string();

                let roles: Vec<_> = item
                    .roles
                    .iter()
                    .map(|role| {
                        let expanded = match role {
                            Role::DatabaseTable => {
                                let database_table =
                                    DatabaseTableModel::new(item.item.clone(), false)
                                        .map_err(|err| err.into_syn_error(item.item.span()))?
                                        .expand()?;

                                quote!(awto::schema::Role::DatabaseTable(#database_table))
                            }
                            Role::DatabaseSubTable(parent_ident) => {
                                let parent = self
                                    .items
                                    .iter()
                                    .find(|item| item.item.ident == *parent_ident)
                                    .ok_or_else(|| {
                                        syn::Error::new(parent_ident.span(), "parent not found")
                                    })?;

                                let database_table =
                                    DatabaseTableModel::new(parent.item.clone(), false)
                                        .map_err(|err| err.into_syn_error(item.item.span()))?
                                        .expand()?;

                                quote!(awto::schema::Role::DatabaseSubTable(#database_table))
                            }
                            Role::ProtobufMessage => {
                                let protobuf_message = ProtobufMessageModel::new(item.item.clone())
                                    .map_err(|err| err.into_syn_error(item.item.span()))?
                                    .expand()?;

                                quote!(awto::schema::Role::ProtobufMessage(#protobuf_message) )
                            }
                        };

                        Result::<_, syn::Error>::Ok(expanded)
                    })
                    .collect::<Result<_, _>>()?;

                let rust_fields = item.item.fields.iter().map(|field| {
                    let field_ident_string = field.ident.as_ref().unwrap().to_string();
                    let mut field_ty_string = field.ty.to_token_stream().to_string();
                    field_ty_string.retain(|c| c != ' ');

                    quote!(
                        awto::schema::RustField {
                            name: #field_ident_string.to_string(),
                            ty: #field_ty_string.to_string(),
                        }
                    )
                });

                Result::<_, syn::Error>::Ok(quote!(
                    awto::schema::Model {
                        fields: vec![ #( #rust_fields ),* ],
                        name: #item_ident.to_string(),
                        roles: vec![ #( #roles ),* ],
                    }
                ))
            })
            .collect::<Result<_, _>>()?;

        let item_count = self.items.len();

        Ok(quote!(awto::lazy_static::lazy_static! {
            pub static ref MODELS: [awto::schema::Model; #item_count] = [
                #( #models ),*
            ];
        }))
    }

    fn impl_models(&self) -> syn::Result<TokenStream> {
        let model_impls: Vec<_> = self
            .items
            .iter()
            .map(|item| {
                let item_ident = &item.item.ident;

                let role_impls: Vec<_> = item
                    .roles
                    .iter()
                    .map(|role| {
                        let expanded = match role {
                            Role::DatabaseTable => {
                                let database_table =
                                    DatabaseTableModel::new(item.item.clone(), false)
                                        .map_err(|err| err.into_syn_error(item.item.span()))?
                                        .expand()?;

                                quote!(
                                    impl awto::database::IntoDatabaseTable for #item_ident {
                                        fn database_table() -> awto::database::DatabaseTable {
                                            #database_table
                                        }
                                    }
                                )
                            }
                            Role::DatabaseSubTable(_) => {
                                quote!()
                            }
                            Role::ProtobufMessage => {
                                let protobuf_message = ProtobufMessageModel::new(item.item.clone())
                                    .map_err(|err| err.into_syn_error(item.item.span()))?
                                    .expand()?;

                                quote!(
                                    impl awto::protobuf::IntoProtobufMessage for #item_ident {
                                        fn protobuf_message() -> awto::protobuf::ProtobufMessage {
                                            #protobuf_message
                                        }
                                    }
                                )
                            }
                        };

                        Result::<_, syn::Error>::Ok(expanded)
                    })
                    .collect::<Result<_, _>>()?;

                Result::<_, syn::Error>::Ok(quote!(
                    #( #role_impls )*
                ))
            })
            .collect::<Result<_, _>>()?;

        Ok(quote!(#( #model_impls )*))
    }

    fn strip_attributes(&mut self) {
        for item in &mut self.items {
            item.item.attrs.retain(|attr| {
                !attr
                    .parse_meta()
                    .map(|meta| match meta {
                        syn::Meta::Path(path) => path
                            .get_ident()
                            .map(|ident| ident == "database_table" || ident == "protobuf_message")
                            .unwrap_or(false),
                        syn::Meta::List(list) => list
                            .path
                            .get_ident()
                            .map(|ident| ident == "database_sub_table")
                            .unwrap_or(false),
                        _ => false,
                    })
                    .unwrap_or(false)
            });

            for field in &mut item.item.fields {
                field.attrs.retain(|attr| {
                    attr.parse_meta()
                        .map(|meta| match meta {
                            syn::Meta::Path(p) => p.is_ident("awto"),
                            _ => false,
                        })
                        .unwrap_or(false)
                })
            }
        }
    }
}

impl ProcMacro for Schema {
    type Input = Structs;

    fn new(input: Self::Input) -> Result<Self, Error> {
        let items = Self::parse_input(input).map_err(Error::Syn)?;

        Ok(Schema { items })
    }

    fn expand(mut self) -> syn::Result<TokenStream> {
        let models_cosnt = self.parse_models()?;
        let model_impls = self.impl_models()?;

        self.strip_attributes();

        let items = self.items.into_iter().map(|item| item.item);
        let expanded_input = quote!(#( #items )*);

        Ok(TokenStream::from_iter([
            models_cosnt,
            model_impls,
            expanded_input,
        ]))
    }
}
