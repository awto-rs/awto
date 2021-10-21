use std::iter::FromIterator;

use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{error::Error, util::ProcMacro};

pub struct ProtobufService {
    ident: syn::Ident,
    input: syn::ItemImpl,
    items: Vec<syn::ImplItem>,
}

struct MethodInfo<'a> {
    pub name: String,
    pub param: &'a syn::Ident,
    pub returns: &'a syn::Ident,
    pub validators: Vec<TokenStream>,
    pub is_result: bool,
}

impl ProtobufService {
    fn expand_impl_protobuf_service(&self) -> syn::Result<TokenStream> {
        let Self { ident, items, .. } = self;

        let name = ident.to_string();

        let mut validators = Vec::new();
        let mut methods = Vec::new();
        for item in items {
            match item {
                syn::ImplItem::Method(method) => {
                    let MethodInfo {
                        name,
                        param,
                        returns,
                        validators: method_validators,
                        is_result,
                        ..
                    } = self.decode_impl_method(method)?;
                    validators.extend(method_validators);

                    let is_async = method.sig.asyncness.is_some();

                    // Validate parameter
                    let param_type_validator_ident =
                        format_ident!("{}_{}_ParamTypeValidator", ident, name);
                    validators.push(quote_spanned!(
                            param.span()=>
                                #[allow(non_camel_case_types)]
                                trait #param_type_validator_ident: awto::protobuf::IntoProtobufMessage {}
                                impl #param_type_validator_ident for #param {}
                        ));

                    // Validate return type
                    let return_type_validator_ident =
                        format_ident!("{}_{}_ReturnTypeValidator", ident, name);
                    validators.push(quote_spanned!(
                        returns.span()=>
                            #[allow(non_camel_case_types)]
                            trait #return_type_validator_ident: awto::protobuf::IntoProtobufMessage {}
                            impl #return_type_validator_ident for #returns {}
                    ));

                    methods.push(quote!(
                        awto::protobuf::ProtobufMethod {
                            is_async: #is_async,
                            name: #name.to_string(),
                            param: <#param as awto::protobuf::IntoProtobufMessage>::protobuf_message(),
                            returns: <#returns as awto::protobuf::IntoProtobufMessage>::protobuf_message(),
                            returns_result: #is_result,
                        }
                    ))
                }
                _ => continue,
            }
        }

        Ok(quote!(
            impl awto::protobuf::IntoProtobufService for #ident {
                fn protobuf_service() -> awto::protobuf::ProtobufService {
                    awto::protobuf::ProtobufService {
                        methods: vec![ #( #methods, )* ],
                        module_path: module_path!().to_string(),
                        name: #name.to_string(),
                    }
                }
            }

            #( #validators )*
        ))
    }

    fn decode_impl_method<'a>(
        &'a self,
        method: &'a syn::ImplItemMethod,
    ) -> syn::Result<MethodInfo<'a>> {
        let name = method.sig.ident.to_string().to_camel_case();
        let mut validators = Vec::new();

        let mut inputs = method.sig.inputs.iter();
        let self_param = inputs.next().ok_or_else(|| {
            syn::Error::new(
                method.sig.span(),
                "protobuf methods must take &self and one input",
            )
        })?;
        if !matches!(self_param, syn::FnArg::Receiver(_)) {
            return Err(syn::Error::new(
                self_param.span(),
                "the first parameter must be &self",
            ));
        }
        let param = match inputs.next().ok_or_else(|| {
            syn::Error::new(
                method.sig.span(),
                "protobuf methods must take &self and one input",
            )
        })? {
            syn::FnArg::Receiver(r) => {
                return Err(syn::Error::new(
                    r.span(),
                    "protobuf methods must take &self and one input",
                ))
            }
            syn::FnArg::Typed(pat_type) => match &*pat_type.ty {
                syn::Type::Path(type_path) => type_path.path.get_ident().ok_or_else(|| {
                    syn::Error::new(
                        method.sig.span(),
                        "protobuf methods may only accept parameters of structs",
                    )
                })?,
                _ => {
                    return Err(syn::Error::new(
                        method.sig.span(),
                        "protobuf methods may only accept parameters of structs",
                    ))
                }
            },
        };

        let mut is_result = false;
        let returns = match &method.sig.output {
            syn::ReturnType::Default => {
                return Err(syn::Error::new(
                    method.sig.output.span(),
                    "protobuf methods must have a return type",
                ))
            }
            syn::ReturnType::Type(_, ty) => {
                let return_err = || {
                    syn::Error::new_spanned(ty, "protobuf methods must return a struct or Result<T, E> where E: Into<tonic::Status>")
                };
                match &**ty {
                syn::Type::Path(type_path) => {
                    if let Some(returns) = type_path.path.get_ident() {
                        returns
                    } else {
                        let segment = type_path.path.segments.first().unwrap();
                        let segment_ident = segment.ident.to_string().replace(' ', "");
                        if segment_ident != "Result" && segment_ident != "result::Result" && segment_ident != "std::result::Result" || segment.arguments.is_empty() {
                            return Err(return_err());
                        }
                        is_result = true;
                        match &segment.arguments {
                            syn::PathArguments::AngleBracketed(inner) => {
                                let mut args = inner.args.iter();
                                let first = args.next().ok_or_else(return_err)?;
                                let second = args.next().ok_or_else(return_err)?;
                                if args.next().is_some() {
                                    return Err(return_err());
                                }

                                let first = match first {
                                    syn::GenericArgument::Type(syn::Type::Path(type_path)) => type_path.path.get_ident().ok_or_else(return_err)?,
                                    _ => return Err(return_err()),
                                };
                                let second = match second {
                                    syn::GenericArgument::Type(syn::Type::Path(type_path)) => type_path.path.get_ident().ok_or_else(return_err)?,
                                    _ => return Err(return_err()),
                                };
                                
                                let return_type_result_validator_ident = format_ident!("{}_{}_ReturnTypeResultValidator", self.ident, name);
                                validators.push(quote_spanned!(
                                    second.span()=>
                                        #[allow(non_camel_case_types)]
                                        trait #return_type_result_validator_ident: ::std::convert::Into<::tonic::Status> {}
                                        impl #return_type_result_validator_ident for #second {}
                                ));

                                first
                            },
                            syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => {
                                return Err(return_err());
                            },
                        }
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "protobuf methods must return a struct or Result<T, E> where E: Into<tonic::Status>",
                    ))
                }
            }
            }
        };

        Ok(MethodInfo {
            name,
            param,
            returns,
            validators,
            is_result,
        })
    }
}

impl ProcMacro for ProtobufService {
    type Input = syn::ItemImpl;

    fn new(input: Self::Input) -> Result<Self, Error> {
        let ident = match &*input.self_ty {
            syn::Type::Path(type_path) => type_path.path.get_ident().unwrap().clone(),
            _ => {
                return Err(Error::Syn(syn::Error::new(
                    input.impl_token.span,
                    "impl must be on a struct",
                )))
            }
        };

        let items = input.items.clone();

        Ok(ProtobufService {
            ident,
            input,
            items,
        })
    }

    fn expand(self) -> syn::Result<TokenStream> {
        let input = &self.input;

        let expanded_input = quote!(#input);
        let expanded_impl_protobuf_service = self.expand_impl_protobuf_service()?;

        Ok(TokenStream::from_iter([
            expanded_input,
            expanded_impl_protobuf_service,
        ]))
    }
}
