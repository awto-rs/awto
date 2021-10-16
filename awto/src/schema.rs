use awto_schema::{prelude::DatabaseSchema, protobuf::ProtobufSchema};

pub trait Schema {
    fn database_schemas() -> Vec<DatabaseSchema> {
        Vec::new()
    }

    fn protobuf_schemas() -> Vec<ProtobufSchema> {
        Vec::new()
    }
}

#[macro_export]
macro_rules! register_schemas {
    ($( $name: ident ),*) => {
        pub struct Schema;

        impl ::awto::schema::Schema for Schema {
            fn database_schemas() -> Vec<::awto_schema::database::DatabaseSchema> {
                vec![ $( <$name as ::awto_schema::database::IntoDatabaseSchema>::database_schema(), )* ]
            }

            fn protobuf_schemas() -> Vec<::awto_schema::protobuf::ProtobufSchema> {
                vec![ $( <$name as ::awto_schema::protobuf::IntoProtobufSchema>::protobuf_schema(), )* ]
            }
        }
    };
}
