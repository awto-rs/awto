// This file is automatically @generated by awto-cli v0.1.0

pub use sea_orm;

include!(concat!(env!("OUT_DIR"), "/app.rs"));

/// Product database model
pub mod product {
    sea_orm::include_model!("product");
}