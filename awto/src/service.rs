use crate::protobuf::ProtobufService;

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
            fn protobuf_services() -> Vec<::awto::protobuf::ProtobufService> {
                vec![ $( <$name as ::awto::protobuf::IntoProtobufService>::protobuf_service(), )* ]
            }
        }
    };
}
