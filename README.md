<div align="center">
  <h1>awto</h1>

  <p>
    <strong>Awtomate your 🦀 microservices with awto</strong>
  </p>

[![crate](https://img.shields.io/crates/v/awto.svg)](https://crates.io/crates/awto)

</div>

## What is awto?

Awto treats your rust project as the source of truth for microservices, and generates **database tables** and **protobufs** based on your schema and service.

#### Database Generation

- Database tables are generated from your Rust structs
- Changes to your structs are detected and will modify the table schema accordingly

#### Protobuf Generation

- Generate protobuf schemas from your models
- Compile a protobuf server from your app's service

## Concepts

With awto, you create a [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) for each microservice. Under your workspace, you create two libs: **`schema`** and **`service`**

#### Schema

The schema lib's purpose is to define your microservice models which will be used to generate database tables.

```rust
// schema/src/lib.rs

register_schemas!(Product);

#[derive(Model)]
pub struct Product {
    pub id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub name: String,
    #[awto(max_len = 120)]
    pub description: Option<String>,
    #[awto(default = 0)]
    pub price: i64,
}
```

_See example schema in [`examples/ecom`](examples/ecom/schema/src/lib.rs)._

#### Service

The service lib is where you write your business logic. This business logic can later be used to create a protobuf API _(and in the future a graphql API)_.

```rust
// service/src/lib.rs

register_services!(ProductService);

pub struct ProductService {
    pub conn: DatabaseConnection,
}

#[protobuf_service]
impl ProductService {
    pub async fn find_by_id(&self, request: ProductId) -> Result<Product, Status> {
        let product = product::Entity::find_by_id(request.id)
            .one(&self.conn)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("product not found"))?;

        Ok(product.into())
    }
}
```

_See example service in [`examples/ecom`](examples/ecom/service/src/product.rs)._

#### Cargo workspace

Your schema and service libs should be under a cargo workspace.

```toml
# root Cargo.toml
[workspace]
members = ["schema", "service"]
```

_See example project in [`examples/ecom`](examples/ecom)._

## Awto CLI

Awto provides a cli for generating additional libraries with the `awto compile <lib>` command.
Currently the available libraries are:

- **`database`** - based on [SeaORM](https://github.com/SeaQL/sea-orm), provides a database library for use in your service
- **`protobuf`** - based on [tonic](https://github.com/hyperium/tonic), provides a protobuf server library

## Roadmap

Awto is still in alpha stages and is made mostly as an experiment at this point.
If it gets some attention, serious effort will be put into it to make it a meaningful tool for the Rust community.

## Contributing

Whether you want to share ideas, bugs, suggestions, or other, your contributions to this project are welcomed 🤌

## License

By contributing, you agree that your contributions will be licensed under this repository's [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) License.
