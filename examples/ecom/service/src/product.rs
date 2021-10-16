use awto_schema::macros::protobuf_service;
use chrono::{FixedOffset, Local};
use schema::*;
use tonic::Status;
use uuid::Uuid;

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
                    price: 69,
                    description: None,
                    category: None,
                }],
            })
        } else if request.id == "2" {
            Ok(ProductList {
                products: vec![Product {
                    id: Uuid::new_v4(),
                    created_at: Local::now().with_timezone(&FixedOffset::east(0)),
                    updated_at: Local::now().with_timezone(&FixedOffset::east(0)),
                    name: "2".to_string(),
                    price: 420,
                    description: None,
                    category: None,
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
    use awto_schema::protobuf::IntoProtobufService;

    #[test]
    fn service_name() {
        assert_eq!(ProductService::protobuf_service().name, "ProductService");
    }

    #[test]
    fn methods() {
        let methods = ProductService::protobuf_service().methods;
        assert_eq!(methods.len(), 1);

        let create_product_method = methods.get(0).unwrap();
        assert_eq!(create_product_method.name, "FindProduct");
    }
}
