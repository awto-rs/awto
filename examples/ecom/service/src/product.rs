use awto::macros::protobuf_service;
use database::{
    product,
    sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveValue},
};
use schema::*;
use tonic::Status;

pub struct ProductService {
    pub conn: database::sea_orm::DatabaseConnection,
}

#[protobuf_service]
impl ProductService {
    pub async fn create_product(&self, request: NewProduct) -> Result<ProductId, Status> {
        let active_model = product::ActiveModel {
            name: request.name.into_active_value(),
            price: request.price.unwrap_or_default().into_active_value(),
            description: request.description.into_active_value(),
            category: request.category.into_active_value(),
            ..Default::default()
        };

        let result = active_model
            .insert(&self.conn)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(ProductId {
            id: result.id.unwrap(),
        })
    }

    pub async fn find_product(&self, request: ProductId) -> Result<Product, Status> {
        let product = product::Entity::find_by_id(request.id)
            .one(&self.conn)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("product not found"))?;

        Ok(product.into())
    }

    pub async fn list_products(&self, _request: Empty) -> Result<ProductList, Status> {
        let products: Vec<Product> = product::Entity::find()
            .all(&self.conn)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .into_iter()
            .map(|product| product.into())
            .collect();

        Ok(ProductList { products })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use awto::protobuf::IntoProtobufService;

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
