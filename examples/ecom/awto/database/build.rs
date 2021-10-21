use std::{env, error};

use awto::schema::Role;
use awto_compile::database::compile_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    dotenv::dotenv().ok();

    let pg_schema = env::var("DATABASE_SCHEMA").unwrap_or_else(|_| "public".to_string());
    let uri = env::var("DATABASE_URL").expect("missing env DATABASE_URL");

    compile_database(&uri, schema::MODELS.to_vec()).await?;

    sea_orm_build::generate_models(
        &pg_schema,
        &uri,
        &schema::MODELS.iter().fold(Vec::new(), |mut acc, model| {
            for role in &model.roles {
                if let Role::DatabaseTable(table) = role {
                    acc.push(table.name.as_str())
                }
            }

            acc
        }),
    )
    .await?;

    Ok(())
}
