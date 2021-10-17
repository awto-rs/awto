use crate::database::DatabaseSchema;
use crate::protobuf::ProtobufSchema;

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
            fn database_schemas() -> Vec<::awto::database::DatabaseSchema> {
                vec![ $( <$name as ::awto::database::IntoDatabaseSchema>::database_schema(), )* ]
            }

            fn protobuf_schemas() -> Vec<::awto::protobuf::ProtobufSchema> {
                vec![ $( <$name as ::awto::protobuf::IntoProtobufSchema>::protobuf_schema(), )* ]
            }
        }
    };
}
