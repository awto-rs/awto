use awto_schema::*;

#[derive(Model)]
pub struct Product {
    pub name: String,
    #[awto(default = 0)]
    pub price: u64,
    #[awto(max_len = 256)]
    pub description: Option<String>,
}
