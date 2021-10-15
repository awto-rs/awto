use awto_schema::{
    database::DatabaseSchema,
    protobuf::{ProtobufSchema, ProtobufService},
};

pub trait AwtoApp {
    fn app_name() -> &'static str;

    fn app_config() -> AppConfig {
        AppConfig::default()
    }

    fn database_schemas() -> &'static [&'static dyn DatabaseSchema] {
        &[]
    }

    fn protobuf_schemas() -> &'static [&'static dyn ProtobufSchema] {
        &[]
    }

    fn protobuf_services() -> &'static [&'static dyn ProtobufService] {
        &[]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppConfig {
    pub compile_database: bool,
    pub compile_protobuf: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            compile_database: true,
            compile_protobuf: true,
        }
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
    ($( $i: ident ),*) => {
        ::awto::database_schemas!(schema: $( $i ),*)
    };
    ($module: ident : $( $i: ident ),*) => {
        &[
            $(
                ::awto::__paste! {
                    &crate::$module::[<$i DatabaseSchema>]
                }
            ),*
        ]
    };
}

#[macro_export]
macro_rules! protobuf_schemas {
    ($( $i: ident ),*) => {
        ::awto::protobuf_schemas!(schema: $( $i ),*)
    };
    ($module: ident : $( $i: ident ),*) => {
        &[
            $(
                ::awto::__paste! {
                    &crate::$module::[<$i ProtobufSchema>]
                }
            ),*
        ]
    };
}

#[macro_export]
macro_rules! protobuf_services {
    ($( $i: ident ),*) => {
        ::awto::protobuf_services!(service: $( $i ),*)
    };
    ($module: ident : $( $i: ident ),*) => {
        &[
            $(
                &crate::$module::$i
            ),*
        ]
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

            fn database_schemas() -> &'static [&'static dyn ::awto_schema::database::DatabaseSchema] {
                ::awto::database_schemas!($( $schemas ),*)
            }

            fn protobuf_schemas() -> &'static [&'static dyn ::awto_schema::protobuf::ProtobufSchema] {
                ::awto::protobuf_schemas!($( $schemas ),*)
            }

            fn protobuf_services() -> &'static [&'static dyn ::awto_schema::protobuf::ProtobufService] {
                ::awto::protobuf_services!($( $services ),*)
            }
        }
    };
}
