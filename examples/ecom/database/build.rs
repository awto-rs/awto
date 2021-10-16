use std::env;

use awto::schema::Schema;
use awto_compile::database::compile_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let uri = env::var("DATABASE_URL").expect("missing env DATABASE_URL");

    compile_database(&uri, schema::Schema::database_schemas()).await?;

    sea_orm_build::generate_models(
        "public",
        &uri,
        &schema::Schema::database_schemas()
            .iter()
            .map(|schema| schema.table_name.as_str())
            .collect::<Vec<_>>(),
    )
    .await?;

    Ok(())
}
