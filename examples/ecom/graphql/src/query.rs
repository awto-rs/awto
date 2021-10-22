use async_graphql::*;
use schema::Empty;

pub struct ProductQuery {
    pub service: service::product::ProductService,
}

#[derive(InputObject, Debug, Clone)]
pub struct ProductIdInput {
    pub id: awto::prelude::Uuid,
}

impl From<ProductIdInput> for schema::ProductId {
    fn from(val: ProductIdInput) -> Self {
        Self { id: val.id }
    }
}

impl From<schema::ProductId> for ProductIdInput {
    fn from(val: schema::ProductId) -> Self {
        Self { id: val.id }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ProductId {
    pub id: awto::prelude::Uuid,
}

impl From<ProductId> for schema::ProductId {
    fn from(val: ProductId) -> Self {
        Self { id: val.id }
    }
}

impl From<schema::ProductId> for ProductId {
    fn from(val: schema::ProductId) -> Self {
        Self { id: val.id }
    }
}

#[derive(InputObject, Debug, Clone)]
pub struct ProductInput {
    pub id: awto::prelude::Uuid,
    pub created_at: awto::prelude::DateTime<awto::prelude::FixedOffset>,
    pub updated_at: awto::prelude::DateTime<awto::prelude::FixedOffset>,
    pub name: String,
    pub price: i64,
    pub description: Option<String>,
    pub category: Option<String>,
}

impl From<ProductInput> for schema::Product {
    fn from(val: ProductInput) -> Self {
        Self {
            id: val.id,
            created_at: val.created_at,
            updated_at: val.updated_at,
            name: val.name,
            price: val.price,
            description: val.description,
            category: val.description,
        }
    }
}

impl From<schema::Product> for ProductInput {
    fn from(val: schema::Product) -> Self {
        Self {
            id: val.id,
            created_at: val.created_at,
            updated_at: val.updated_at,
            name: val.name,
            price: val.price,
            description: val.description,
            category: val.description,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Product {
    pub id: awto::prelude::Uuid,
    pub created_at: awto::prelude::DateTime<awto::prelude::FixedOffset>,
    pub updated_at: awto::prelude::DateTime<awto::prelude::FixedOffset>,
    pub name: String,
    pub price: i64,
    pub description: Option<String>,
    pub category: Option<String>,
}

impl From<Product> for schema::Product {
    fn from(val: Product) -> Self {
        Self {
            id: val.id,
            created_at: val.created_at,
            updated_at: val.updated_at,
            name: val.name,
            price: val.price,
            description: val.description,
            category: val.description,
        }
    }
}

impl From<schema::Product> for Product {
    fn from(val: schema::Product) -> Self {
        Self {
            id: val.id,
            created_at: val.created_at,
            updated_at: val.updated_at,
            name: val.name,
            price: val.price,
            description: val.description,
            category: val.description,
        }
    }
}

#[derive(InputObject, Debug, Clone)]
pub struct ProductListInput {
    pub products: Vec<ProductInput>,
}

impl From<ProductListInput> for schema::ProductList {
    fn from(val: ProductListInput) -> Self {
        Self {
            products: val.products.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<schema::ProductList> for ProductListInput {
    fn from(val: schema::ProductList) -> Self {
        Self {
            products: val.products.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ProductList {
    pub products: Vec<Product>,
}

impl From<ProductList> for schema::ProductList {
    fn from(val: ProductList) -> Self {
        Self {
            products: val.products.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<schema::ProductList> for ProductList {
    fn from(val: schema::ProductList) -> Self {
        Self {
            products: val.products.into_iter().map(Into::into).collect(),
        }
    }
}

#[Object]
impl ProductQuery {
    async fn find_product(&self, request: ProductIdInput) -> Result<Product> {
        let resp = self
            .service
            .find_product(request.into())
            .await
            .map_err(|status| {
                async_graphql::Error::new(status.message()).extend_with(|_, e| {
                    e.set("status", format!("{:?}", status.code()));
                    e.set("status_message", status.code().to_string());
                })
            })?;

        Ok(resp.into())
    }

    async fn list_products(&self) -> Result<ProductList> {
        let resp = self
            .service
            .list_products(Empty {})
            .await
            .map_err(|status| {
                async_graphql::Error::new(status.message()).extend_with(|_, e| {
                    e.set("status", format!("{:?}", status.code()));
                    e.set("status_message", status.code().to_string());
                })
            })?;

        Ok(resp.into())
    }
}
