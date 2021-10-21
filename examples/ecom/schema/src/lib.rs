use awto::prelude::*;

schema! {
    #[database_table]
    #[protobuf_message]
    pub struct Product {
        pub id: Uuid,
        pub created_at: DateTime<FixedOffset>,
        pub updated_at: DateTime<FixedOffset>,
        pub name: String,
        #[awto(default = 0)]
        pub price: i64,
        #[awto(max_len = 120)]
        pub description: Option<String>,
        pub category: Option<String>,
    }

    #[protobuf_message]
    pub struct Empty {}

    #[protobuf_message]
    pub struct ProductId {
        pub id: Uuid,
    }

    #[protobuf_message]
    #[database_sub_table(Product)]
    pub struct NewProduct {
        pub name: String,
        pub price: Option<i64>,
        pub description: Option<String>,
        pub category: Option<String>,
    }

    #[protobuf_message]
    pub struct ProductList {
        pub products: Vec<Product>,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn models() {
        println!("{:#?}", &*MODELS);
    }
}
