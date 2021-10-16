use proto::product_service_client::ProductServiceClient;
use proto::ProductId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ProductServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ProductId {
        id: "1".to_string(),
    });

    let response = client.find_product(request).await?;

    println!("{:#?}", response);

    Ok(())
}
