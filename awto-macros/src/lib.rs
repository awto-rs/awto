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
mod error;
mod proc_macros;
mod util;

#[proc_macro_attribute]
pub fn protobuf_service(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(
        input as <crate::proc_macros::ProtobufService as crate::util::ProcMacro>::Input
    );

    let impl_span = input.impl_token.span;

    match <crate::proc_macros::ProtobufService as crate::util::ProcMacro>::new(input) {
        Ok(proc) => crate::util::ProcMacro::expand(proc)
            .unwrap_or_else(syn::Error::into_compile_error)
            .into(),
        Err(err) => err.into_compile_error(Some(impl_span)),
    }
}

#[proc_macro]
pub fn schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(
        input as <crate::proc_macros::Schema as crate::util::ProcMacro>::Input
    );

    match <crate::proc_macros::Schema as crate::util::ProcMacro>::new(input) {
        Ok(proc) => crate::util::ProcMacro::expand(proc)
            .unwrap_or_else(syn::Error::into_compile_error)
            .into(),
        Err(err) => err.into_compile_error(None),
    }
}
