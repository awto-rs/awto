use crate as awto;
use crate::prelude::*;
use chrono::Local;
use tonic::Status;

schema! {
    #[database_table]
    #[protobuf_message]
    pub struct Product {
        pub id: Uuid,
        pub created_at: DateTime<FixedOffset>,
        pub updated_at: DateTime<FixedOffset>,
        pub name: String,
        #[awto(default = 0)]
        pub price: i64,
        #[awto(max_len = 120)]
        pub description: Option<String>,
    }

    #[protobuf_message]
    pub struct ProductId {
        pub id: Uuid,
    }

    #[protobuf_message]
    pub struct ProductList {
        pub products: Vec<Product>,
    }

    #[protobuf_message]
    #[database_sub_table(Product)]
    pub struct NewProduct {
        pub name: String,
        pub price: Option<i64>,
        pub description: Option<String>,
    }
}

#[derive(Default)]
pub struct ProductService;

#[protobuf_service]
impl ProductService {
    pub fn find_product(&self, request: ProductId) -> Result<ProductList, Status> {
        if request.id == Uuid::default() {
            Ok(ProductList {
                products: vec![Product {
                    id: Uuid::default(),
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
    use super::*;

    #[test]
    fn models() {
        println!("{:#?}", &*MODELS);
    }
}
