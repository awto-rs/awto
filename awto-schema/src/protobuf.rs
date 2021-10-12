use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEFAULT_PROTOBUF_FIELDS: [ProtobufField; 3] = [
        ProtobufField {
            name: "id".to_string(),
            ty: "string".to_string(),
            required: true,
        },
        ProtobufField {
            name: "created_at".to_string(),
            ty: "google.protobuf.Timestamp".to_string(),
            required: true,
        },
        ProtobufField {
            name: "updated_at".to_string(),
            ty: "google.protobuf.Timestamp".to_string(),
            required: true,
        },
    ];
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtobufField {
    pub name: String,
    pub ty: String,
    pub required: bool,
}

pub trait IntoProtobufSchema {
    type Schema: ProtobufSchema + Default;

    fn protobuf_schema() -> Self::Schema {
        Self::Schema::default()
    }
}

pub trait ProtobufSchema {
    fn message_name(&self) -> &'static str;

    fn fields(&self) -> Vec<ProtobufField>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate as awto_schema;
    use crate::*;

    #[derive(Model)]
    pub struct Product {
        pub name: String,
        #[awto(default = 0)]
        pub price: u64,
        #[awto(max_len = 256)]
        pub description: Option<String>,
    }

    #[test]
    fn message_name() {
        assert_eq!(Product::protobuf_schema().message_name(), "Product");
    }

    #[test]
    fn columns() {
        let fields = Product::protobuf_schema().fields();
        let expected = vec![
            ProtobufField {
                name: "id".to_string(),
                ty: "string".to_string(),
                required: true,
            },
            ProtobufField {
                name: "created_at".to_string(),
                ty: "google.protobuf.Timestamp".to_string(),
                required: true,
            },
            ProtobufField {
                name: "updated_at".to_string(),
                ty: "google.protobuf.Timestamp".to_string(),
                required: true,
            },
            ProtobufField {
                name: "name".to_string(),
                ty: "string".to_string(),
                required: true,
            },
            ProtobufField {
                name: "price".to_string(),
                ty: "uint64".to_string(),
                required: true,
            },
            ProtobufField {
                name: "description".to_string(),
                ty: "string".to_string(),
                required: false,
            },
        ];
        assert_eq!(fields, expected);
    }
}
