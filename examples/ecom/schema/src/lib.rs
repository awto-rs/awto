use awto_schema::prelude::*;
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

awto::register_schemas!(Product);

#[derive(Model)]
pub struct Product {
    pub id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub name: String,
    #[awto(default = 0)]
    pub price: u64,
    #[awto(max_len = 120)]
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(ProtobufModel)]
pub struct ProductId {
    pub id: String,
}

#[derive(ProtobufModel)]
pub struct ProductList {
    pub products: Vec<Product>,
}
