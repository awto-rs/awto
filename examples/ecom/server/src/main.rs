use proto::product_service_server::ProductServiceServer;
use service::product::ProductService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let product_service = ProductService::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(ProductServiceServer::new(product_service))
        .serve(addr)
        .await?;

    Ok(())
}
