use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::quote_spanned;

use crate::error::Error;

use super::{DeriveDatabaseModel, DeriveProtobufModel};

pub struct DeriveModel {
    database_model: DeriveDatabaseModel,
    protobuf_model: DeriveProtobufModel,
}

impl DeriveModel {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let database_model = DeriveDatabaseModel::new(input.clone())?;
        let protobuf_model = DeriveProtobufModel::new(input)?;

        Ok(DeriveModel {
            database_model,
            protobuf_model,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_derive_database_model = self.database_model.expand()?;
        let expanded_derive_protobuf_model = self.protobuf_model.expand()?;

        Ok(TokenStream::from_iter([
            expanded_derive_database_model,
            expanded_derive_protobuf_model,
        ]))
    }
}

pub fn expand_derive_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveModel::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
