use awto::prelude::*;
use chrono::{DateTime, FixedOffset, Local};
use tonic::Status;
use uuid::Uuid;

#[derive(Model)]
pub struct Product {
    pub id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub name: String,
    #[awto(default = 0)]
    pub price: i64,
    #[awto(max_len = 256)]
    pub description: Option<String>,
}

#[derive(Model)]
pub struct Variant {
    pub id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[awto(references = (Product, "id"))]
    pub product_id: Uuid,
    pub name: String,
    pub price: i64,
}

#[derive(ProtobufModel)]
pub struct ProductId {
    pub id: String,
}

#[derive(ProtobufModel)]
pub struct ProductList {
    pub products: Vec<Product>,
}

#[derive(Default)]
pub struct ProductService;

#[protobuf_service]
impl ProductService {
    pub fn find_product(&self, request: ProductId) -> Result<ProductList, Status> {
        if request.id == "1" {
            Ok(ProductList {
                products: vec![Product {
                    id: Uuid::new_v4(),
                    created_at: Local::now().with_timezone(&FixedOffset::east(0)),
                    updated_at: Local::now().with_timezone(&FixedOffset::east(0)),
                    name: "1".to_string(),
                    price: 20,
                    description: None,
                }],
            })
        } else {
            Err(Status::not_found("resouce not found"))
        }
    }
}

#[cfg(test)]
mod test {
    use awto::database::IntoDatabaseSchema;

    use super::*;

    #[test]
    fn variant() {
        assert_eq!(
            Variant::database_schema()
                .columns
                .iter()
                .find(|col| col.name == "product_id")
                .unwrap()
                .references,
            Some(("product".to_string(), "id".to_string()))
        );
    }
}
