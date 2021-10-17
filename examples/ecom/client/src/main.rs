use protobuf::product_service_client::ProductServiceClient;
use protobuf::{Empty, NewProduct, ProductId};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ProductServiceClient::connect("http://[::1]:50051").await?;

    let new_product = NewProduct {
        name: "My Cool Product".to_string(),
        price: Some(25),
        description: Some("My amazing new product".to_string()),
        category: Some("toys".to_string()),
    };
    let create_product_resp = client
        .create_product(Request::new(new_product))
        .await?
        .into_inner();
    println!("Created product with ID:\n  {}\n", create_product_resp.id);

    let product_id = ProductId {
        id: create_product_resp.id,
    };
    let find_product_resp = client
        .find_product(Request::new(product_id))
        .await?
        .into_inner();
    println!("Fetched product:\n  {:#?}\n", find_product_resp);

    let empty = Empty {};
    let list_products_resp = client
        .list_products(Request::new(empty))
        .await?
        .into_inner();
    println!("Listed product:\n  {:#?}", list_products_resp);

    Ok(())
}
