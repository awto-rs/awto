use awto::prelude::*;
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

register_schemas!(Product);

#[derive(Model)]
pub struct Product {
    pub id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub name: String,
    #[awto(default = 0)]
    pub price: i64,
    #[awto(max_len = 120)]
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(ProtobufModel)]
pub struct Empty {}

#[derive(ProtobufModel)]
pub struct ProductId {
    pub id: Uuid,
}

#[derive(ProtobufModel)]
pub struct NewProduct {
    pub name: String,
    pub price: Option<i64>,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(ProtobufModel)]
pub struct ProductList {
    pub products: Vec<Product>,
}
