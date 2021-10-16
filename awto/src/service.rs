use awto_schema::protobuf::ProtobufService;

pub trait Service {
    fn protobuf_services() -> Vec<ProtobufService> {
        Vec::new()
    }
}

#[macro_export]
macro_rules! register_services {
    ($( $name: ident ),*) => {
        pub struct Service;

        impl ::awto::service::Service for Service {
            fn protobuf_services() -> Vec<::awto_schema::protobuf::ProtobufService> {
                vec![ $( <$name as ::awto_schema::protobuf::IntoProtobufService>::protobuf_service(), )* ]
            }
        }
    };
}
