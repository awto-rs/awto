//! <div align="center">
//!   <h1>awto</h1>
//!
//!   <p>
//!     <strong>Awtomate your 🦀 microservices with awto</strong>
//!   </p>
//!
//! </div>
//!
//! # awto-macros
//!
//! This crate provides macros used by [`awto`](https://docs.rs/awto).
//!
//! See more on the [repository](https://github.com/awto-rs/awto).

mod attributes;
mod derives;
mod error;
mod proc_macros;
mod util;

macro_rules! create_derive {
    ($name:ident $(, $( $attrs:ident ),+)?) => {
        paste::paste! {
            #[proc_macro_derive($name, attributes($( $( $attrs ),*)?))]
            pub fn [< $name:snake >](input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let input = syn::parse_macro_input!(input as syn::DeriveInput);
                let ident_span = input.ident.span();

                match <crate::derives::[< Derive $name >] as crate::util::DeriveMacro>::new(input) {
                    Ok(derive) => crate::util::DeriveMacro::expand(&derive)
                        .unwrap_or_else(syn::Error::into_compile_error)
                        .into(),
                    Err(err) => err.into_compile_error(ident_span),
                }
            }
        }
    }
}

create_derive!(Model, awto);
create_derive!(DatabaseModel, awto);
create_derive!(ProtobufModel, awto);

#[proc_macro_attribute]
pub fn protobuf_service(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let input = syn::parse_macro_input!(
        input as <crate::proc_macros::ProtobufService as crate::util::ProcMacro>::Input
    );

    let impl_span = input.impl_token.span;

    match <crate::proc_macros::ProtobufService as crate::util::ProcMacro>::new(args, input) {
        Ok(proc) => crate::util::ProcMacro::expand(&proc)
            .unwrap_or_else(syn::Error::into_compile_error)
            .into(),
        Err(err) => err.into_compile_error(impl_span),
    }
}
