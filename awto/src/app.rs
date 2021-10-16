use crate::{schema::Schema, service::Service};

pub trait AwtoApp {
    type Schema: Schema;
    type Service: Service;

    const APP_NAME: &'static str;

    fn app_config() -> AppConfig {
        AppConfig::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppConfig {}

impl Default for AppConfig {
    fn default() -> Self {
        Self {}
    }
}

#[macro_export]
macro_rules! app_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

#[macro_export]
macro_rules! database_schemas {
    ($( $name: ident ),*) => {
        ::awto::database_schemas!(schema: $( $name ),*)
    };
    ($module: ident : $( $name: ident ),*) => {
        vec![ $( <crate::$module::$name as ::awto_schema::database::IntoDatabaseSchema>::database_schema() ),* ]
    };
}

#[macro_export]
macro_rules! protobuf_schemas {
    ($( $name: ident ),*) => {
        ::awto::protobuf_schemas!(schema: $( $name ),*)
    };
    ($module: ident : $( $name: ident ),*) => {
        vec![ $( <crate::$module::$name as ::awto_schema::protobuf::IntoProtobufSchema>::protobuf_schema() ),* ]
    };
}

#[macro_export]
macro_rules! protobuf_services {
    ($( $name: ident ),*) => {
        ::awto::protobuf_services!(service: $( $name ),*)
    };
    ($module: ident : $( $name: ident ),*) => {
        vec![ $( <crate::$module::$name as ::awto_schema::protobuf::IntoProtobufService>::protobuf_service() ),* ]
    };
}

#[macro_export(local_inner_macros)]
macro_rules! app {
    (schemas = ($( $schemas: ident ),*), services = ($( $services: ident ),*)) => {
        app!(name = app_name!(), config = ::awto::AppConfig::default(), schemas = ($( $schemas ),*), services = ($( $services ),*));
    };
    (name = $name: expr, schemas = ($( $schemas: ident ),*), services = ($( $services: ident ),*)) => {
        app!(name = $name, config = ::awto::AppConfig::default(), schemas = ($( $schemas ),*), services = ($( $services ),*));
    };
    (config = $app_config: stmt, schemas = ($( $schemas: ident ),*), services = ($( $services: ident ),*)) => {
        app!(name = app_name!(), config = $app_config, schemas = ($( $schemas ),*), services = ($( $services ),*));
    };
    (name = $name: expr, config = $app_config: stmt, schemas = ($( $schemas: ident ),*), services = ($( $services: ident ),*)) => {
        pub struct App;

        impl ::awto::AwtoApp for App {
            fn app_name() -> &'static str {
                $name
            }

            fn app_config() -> ::awto::AppConfig {
                $app_config
            }

            fn database_schemas() -> Vec<::awto_schema::database::DatabaseSchema> {
                ::awto::database_schemas!($( $schemas ),*)
            }

            fn protobuf_schemas() -> Vec<::awto_schema::protobuf::ProtobufSchema> {
                ::awto::protobuf_schemas!($( $schemas ),*)
            }

            fn protobuf_services() -> Vec<::awto_schema::protobuf::ProtobufService> {
                ::awto::protobuf_services!($( $services ),*)
            }
        }
    };
}
