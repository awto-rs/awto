use awto_schema::{database::DatabaseSchema, protobuf::ProtobufSchema};

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
macro_rules! app {
    ($( $i: ident ),*) => {
        pub struct App;

        impl ::awto::AwtoApp for App {
            fn app_name() -> &'static str {
                ::awto::app_name!()
            }

            fn app_config() -> ::awto::AppConfig {
                ::awto::AppConfig::default()
            }

            fn database_schemas() -> &'static [&'static dyn ::awto_schema::database::DatabaseSchema] {
                ::awto::database_schemas!($( $i ),*)
            }

            fn protobuf_schemas() -> &'static [&'static dyn ::awto_schema::protobuf::ProtobufSchema] {
                ::awto::protobuf_schemas!($( $i ),*)
            }
        }
    };
    ($app_config: stmt, $( $i: ident ),*) => {
        pub struct App;

        impl ::awto::AwtoApp for App {
            fn app_name() -> &'static str {
                ::awto::app_name!()
            }

            fn app_config() -> ::awto::AppConfig {
                $app_config
            }

            fn database_schemas() -> &'static [&'static dyn ::awto_schema::database::DatabaseSchema] {
                ::awto::database_schemas!($( $i ),*)
            }

            fn protobuf_schemas() -> &'static [&'static dyn ::awto_schema::protobuf::ProtobufSchema] {
                ::awto::protobuf_schemas!($( $i ),*)
            }
        }
    };
    ($name: literal, $app_config: stmt, $( $i: ident ),*) => {
        pub struct App;

        impl ::awto::AwtoApp for App {
            fn app_name() -> &'static str {
                $name
            }

            fn app_config() -> ::awto::AppConfig {
                $app_config
            }

            fn database_schemas() -> &'static [&'static dyn ::awto_schema::database::DatabaseSchema] {
                ::awto::database_schemas!($( $i ),*)
            }

            fn protobuf_schemas() -> &'static [&'static dyn ::awto_schema::protobuf::ProtobufSchema] {
                ::awto::protobuf_schemas!($( $i ),*)
            }
        }
    };
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
