use std::env;

use protobuf::product_service_server::ProductServiceServer;
use service::product::ProductService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let addr = "[::1]:50051".parse().unwrap();
    let conn =
        sea_orm::Database::connect(env::var("DATABASE_URL").expect("missing env DATABASE_URL"))
            .await?;
    let product_service = ProductService { conn };

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(ProductServiceServer::new(product_service))
        .serve(addr)
        .await?;

    Ok(())
}
