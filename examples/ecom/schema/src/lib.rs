use awto_schema::*;
use uuid::Uuid;

#[derive(Model)]
pub struct Product {
    pub name: String,
    #[awto(default = 0)]
    pub price: u64,
    #[awto(max_len = 120)]
    pub description: Option<String>,
}

#[derive(Model)]
pub struct Variant {
    #[awto(references = ("product", "id"))]
    pub product_id: Uuid,
    pub name: String,
    pub price: u64,
}
