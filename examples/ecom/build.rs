use awto_compile::{database::compile_database, protobuf::compile_protobuf};
use awto_schema::{database::IntoDatabaseSchema, protobuf::IntoProtobufSchema};
use schema::*;
use sqlx::PgPool;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(debug_assertions) {
        let pool = PgPool::connect("postgres://ari@0.0.0.0:5432/product").await?;

        let sql = compile_database(
            &pool,
            &[&Product::database_schema(), &Variant::database_schema()],
        )
        .await?;

        let proto = compile_protobuf(&[&Product::protobuf_schema(), &Variant::protobuf_schema()]);

        fs::write("database.sql", sql).await?;
        fs::write("schema.proto", proto).await?;
    }

    Ok(())
}
