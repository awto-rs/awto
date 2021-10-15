use std::iter::FromIterator;

use proc_macro2::TokenStream;

use crate::{error::Error, util::DeriveMacro};

use super::{DeriveDatabaseModel, DeriveProtobufModel};

pub struct DeriveModel {
    database_model: DeriveDatabaseModel,
    protobuf_model: DeriveProtobufModel,
}

impl DeriveMacro for DeriveModel {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let database_model = DeriveDatabaseModel::new(input.clone())?;
        let protobuf_model = DeriveProtobufModel::new(input)?;

        Ok(DeriveModel {
            database_model,
            protobuf_model,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_derive_database_model = self.database_model.expand()?;
        let expanded_derive_protobuf_model = self.protobuf_model.expand()?;

        Ok(TokenStream::from_iter([
            expanded_derive_database_model,
            expanded_derive_protobuf_model,
        ]))
    }
}
