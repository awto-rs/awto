use better_bae::TryFromAttributes;
use proc_macro2::TokenStream;
use syn::Token;

use crate::error::Error;

pub trait ProcMacro {
    type Input;

    fn new(input: Self::Input) -> Result<Self, Error>
    where
        Self: Sized;

    fn expand(self) -> syn::Result<TokenStream>;
}

pub struct Field<Attr> {
    pub attrs: Attr,
    pub field: syn::Field,
}

pub fn parse_fields<Attr>(
    fields: syn::punctuated::Punctuated<syn::Field, Token![,]>,
) -> Result<Vec<Field<Attr>>, Error>
where
    Attr: Default + TryFromAttributes,
{
    fields
        .into_iter()
        .map(|field| {
            Ok(Field {
                attrs: Attr::try_from_attributes(&field.attrs)
                    .map_err(Error::Syn)?
                    .unwrap_or_default(),
                field,
            })
        })
        .collect::<Result<_, _>>()
}
