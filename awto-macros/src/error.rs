use proc_macro::TokenStream;
use proc_macro2::Span;

pub enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

impl Error {
    pub fn into_compile_error(self, span: Span) -> TokenStream {
        match self {
            Error::InputNotStruct => quote::quote_spanned! {
                span => compile_error!(concat!("you can only derive on structs"));
            }
            .into(),
            Error::Syn(syn_err) => syn::Error::into_compile_error(syn_err).into(),
        }
    }
}
