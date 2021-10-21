use proc_macro::TokenStream;
use proc_macro2::Span;

pub enum Error {
    FieldsNotNamed,
    Syn(syn::Error),
}

impl Error {
    pub fn into_syn_error(self, span: Span) -> syn::Error {
        match self {
            Error::FieldsNotNamed => syn::Error::new(span, "fields must be named"),
            Error::Syn(syn) => syn,
        }
    }

    pub fn into_compile_error(self, span: Option<Span>) -> TokenStream {
        match self {
            Error::FieldsNotNamed => match span {
                Some(span) => quote::quote_spanned! {
                    span => compile_error!(concat!("fields must be named"));
                }
                .into(),
                None => quote::quote!(compile_error!(concat!("fields must be named"))).into(),
            },
            Error::Syn(syn_err) => syn::Error::into_compile_error(syn_err).into(),
        }
    }
}
