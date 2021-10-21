use std::{env, fmt::Write};

use awto::{
    protobuf::{ProtobufField, ProtobufMessage, ProtobufMethod, ProtobufService},
    schema::{Model, Role},
};
use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::util::{is_ty_vec, strip_ty_option};

const COMPILED_PROTO_FILE: &str = "app.proto";
const COMPILED_RUST_FILE: &str = "app.rs";

#[cfg(feature = "async")]
pub fn compile_protobuf(
    models: Vec<Model>,
    services: Vec<ProtobufService>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    let app_config = A::app_config();
    let out_dir = env::var("OUT_DIR").unwrap();

    if !app_config.compile_protobuf {
        return Ok(());
    }

    let compiler = ProtobufCompiler::new(models, services);

    let proto = compiler.compile_file();
    let proto_path = format!("{}/{}", out_dir, COMPILED_PROTO_FILE);
    fs::write(&proto_path, proto + "\n").await?;

    tonic_build::configure().compile(&[&proto_path], &[&out_dir])?;

    let generated_code = compiler.compile_generated_code();
    if !generated_code.is_empty() {
        let rs_path = format!("{}/{}", out_dir, COMPILED_RUST_FILE);
        let mut schema_file = fs::OpenOptions::new().append(true).open(&rs_path).await?;

        schema_file.write(generated_code.as_bytes()).await?;
        schema_file.sync_all().await?;
    }

    Ok(())
}

#[cfg(not(feature = "async"))]
pub fn compile_protobuf(
    models: Vec<Model>,
    services: Vec<ProtobufService>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;

    let out_dir = env::var("OUT_DIR").unwrap();

    let compiler = ProtobufCompiler::new(models, services);

    let proto = compiler.compile_file();
    let proto_path = format!("{}/{}", out_dir, COMPILED_PROTO_FILE);
    fs::write(&proto_path, proto + "\n")?;

    tonic_build::configure().compile(&[&proto_path], &[&out_dir])?;

    let generated_code = compiler.compile_generated_code();
    if !generated_code.is_empty() {
        let rs_path = format!("{}/{}", out_dir, COMPILED_RUST_FILE);
        let mut schema_file = fs::OpenOptions::new().append(true).open(&rs_path)?;

        write!(schema_file, "{}", generated_code)?;
        schema_file.sync_all()?;
    }

    Ok(())
}

/// Compiles a protobuf schema from a slice of [`ProtobufMessage`]s and [`ProtobufService`]s.
///
/// # Examples
///
/// ```
/// # use awto_compile::protobuf::ProtobufCompiler;
/// # use awto::tests_cfg::*;
/// # use awto::protobuf::IntoProtobufService;
/// let compiler = ProtobufCompiler::new(
///     MODELS.to_vec(),
///     vec![ProductService::protobuf_service()],
/// );
/// let protobuf_file = compiler.compile_file();
///
/// assert_eq!(protobuf_file, r#"syntax = "proto3";
///
/// package app;
///
/// import "google/protobuf/timestamp.proto";
///
/// message Product {
///   string id = 1;
///   google.protobuf.Timestamp created_at = 2;
///   google.protobuf.Timestamp updated_at = 3;
///   string name = 4;
///   int64 price = 5;
///   optional string description = 6;
/// }
///
/// message ProductId {
///   string id = 1;
/// }
///
/// message ProductList {
///   repeated Product products = 1;
/// }
///
/// message NewProduct {
///   string name = 1;
///   optional int64 price = 2;
///   optional string description = 3;
/// }
///
/// service ProductService {
///   rpc FindProduct(ProductId) returns (ProductList);
/// }"#);
/// ```
pub struct ProtobufCompiler {
    models: Vec<Model>,
    services: Vec<ProtobufService>,
}

impl ProtobufCompiler {
    /// Creates a new instance of [`ProtobufCompiler`].
    pub fn new(models: Vec<Model>, services: Vec<ProtobufService>) -> ProtobufCompiler {
        ProtobufCompiler { models, services }
    }

    /// Compiles a protobuf file.
    pub fn compile_file(&self) -> String {
        let mut proto = String::new();

        write!(proto, "{}", self.write_protobuf_header()).unwrap();
        writeln!(proto).unwrap();

        for message in self.all_protobuf_messages() {
            writeln!(proto, "{}", self.write_protobuf_message(message)).unwrap();
        }

        for service in &self.services {
            writeln!(proto, "{}", self.write_protobuf_service(service)).unwrap();
        }

        proto.trim().to_string()
    }

    /// Compiles generated Rust code from schemas and services.
    pub fn compile_generated_code(&self) -> String {
        let mut code = String::new();

        write!(
            code,
            r#"
pub enum TryFromProtoError {{
    InvalidUuid,
    MissingField(String),
}}

impl ::std::fmt::Display for TryFromProtoError {{
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {{
        match self {{
            Self::InvalidUuid => write!(f, "invalid uuid"),
            Self::MissingField(field) => write!(f, "missing field '{{}}'", field),
        }}
    }}
}}
        "#
        )
        .unwrap();

        for (model, _) in self.protobuf_messages() {
            let ident = format_ident!("{}", model.name);

            let mut from_rust_fields = Vec::new();
            let mut from_proto_fields = Vec::new();

            for field in &model.fields {
                let field_ident = format_ident!("{}", field.name);
                let field_ident_string = &field.name;

                let ty = strip_ty_option(&field.ty);

                match ty {
                    "chrono::NaiveDateTime"
                    | "NaiveDateTime"
                    | "chrono::DateTime<chrono::FixedOffset>"
                    | "chrono::DateTime<FixedOffset>"
                    | "DateTime<chrono::FixedOffset>"
                    | "DateTime<FixedOffset>" => {
                        from_rust_fields.push(quote!(
                            #field_ident: Some(::prost_types::Timestamp {
                                nanos: val.#field_ident.timestamp_nanos() as i32,
                                seconds: val.#field_ident.timestamp(),
                            })
                        ));
                        from_proto_fields.push(quote!(
                        #field_ident: {
                            let unwrapped_value = val.#field_ident.ok_or_else(|| TryFromProtoError::MissingField(#field_ident_string.to_string()))?;
                            ::chrono::DateTime::from_utc(
                                ::chrono::naive::NaiveDateTime::from_timestamp(
                                    unwrapped_value.seconds,
                                    unwrapped_value.nanos as u32
                                ),
                                ::chrono::FixedOffset::east(0),
                            )
                        }
                    ));
                    }
                    "uuid::Uuid" | "Uuid" => {
                        from_rust_fields.push(quote!(
                            #field_ident: val.#field_ident.to_string()
                        ));
                        from_proto_fields.push(quote!(
                        #field_ident: ::uuid::Uuid::parse_str(&val.#field_ident).map_err(|_| TryFromProtoError::InvalidUuid)?
                    ));
                    }
                    _ => {
                        if is_ty_vec(ty) {
                            from_rust_fields.push(quote!(#field_ident: val.#field_ident.into_iter().map(|v| v.into()).collect()));
                            from_proto_fields.push(quote!(#field_ident: val.#field_ident.into_iter().map(|v| ::std::convert::TryFrom::try_from(v)).collect::<Result<_, _>>()?));
                        } else {
                            from_rust_fields.push(quote!(#field_ident: val.#field_ident.into()));
                            from_proto_fields.push(quote!(#field_ident: val.#field_ident.into()));
                        }
                    }
                }
            }

            let expanded = quote!(
                impl ::std::convert::TryFrom<#ident> for ::schema::#ident {
                    type Error = TryFromProtoError;

                    #[allow(unused_variables)]
                    fn try_from(val: #ident) -> Result<Self, Self::Error> {
                        Ok(Self {
                            #( #from_proto_fields, )*
                        })
                    }
                }

                impl ::std::convert::From<::schema::#ident> for #ident {
                    #[allow(unused_variables)]
                    fn from(val: ::schema::#ident) -> Self {
                        Self {
                            #( #from_rust_fields, )*
                        }
                    }
                }
            );

            write!(code, "{}", expanded.to_string()).unwrap();
        }

        for service in &self.services {
            let ident = format_ident!("{}", service.name);
            let service_path: TokenStream = service.module_path.parse().unwrap();
            let service_server_name = format_ident!("{}_server", service.name.to_snake_case());

            let mut methods = Vec::new();
            for method in &service.methods {
                let name_ident = format_ident!("{}", method.name.to_snake_case());
                let param_ident = format_ident!("{}", method.param.name);
                let returns_ident = format_ident!("{}", method.returns.name);

                let expanded_call_method = if method.returns_result {
                    if method.is_async {
                        quote!(self.#name_ident(param).await?)
                    } else {
                        quote!(self.#name_ident(param)?)
                    }
                } else if method.is_async {
                    quote!(self.#name_ident(param).await)
                } else {
                    quote!(self.#name_ident(param))
                };

                methods.push(quote!(
                    async fn #name_ident(
                        &self,
                        request: ::tonic::Request<#param_ident>,
                    ) -> Result<::tonic::Response<#returns_ident>, ::tonic::Status> {
                        let inner = request.into_inner();
                        let param = ::std::convert::TryInto::try_into(inner).map_err(|err: TryFromProtoError| ::tonic::Status::invalid_argument(err.to_string()))?;
                        let value = #expanded_call_method;
                        Ok(::tonic::Response::new(value.into()))
                    }
                ));
            }

            let expanded = quote!(
                #[::tonic::async_trait]
                impl #service_server_name::#ident for #service_path::#ident {
                    #( #methods )*
                }
            );

            write!(code, "{}", expanded.to_string()).unwrap();
        }

        code.trim().to_string()
    }

    fn protobuf_messages(&self) -> Vec<(&Model, &ProtobufMessage)> {
        self.models.iter().fold(Vec::new(), |mut acc, model| {
            let roles = model
                .roles
                .iter()
                .filter_map(|role| match role {
                    Role::ProtobufMessage(protobuf_message) => Some((model, protobuf_message)),
                    _ => None,
                })
                .collect::<Vec<_>>();

            acc.extend(roles);

            acc
        })
    }

    fn all_protobuf_messages(&self) -> Vec<&ProtobufMessage> {
        self.protobuf_messages()
            .iter()
            .map(|(_, protobuf_message)| *protobuf_message)
            .collect()
    }

    fn write_protobuf_header(&self) -> String {
        let mut proto = String::new();

        writeln!(proto, r#"syntax = "proto3";"#).unwrap();
        writeln!(proto).unwrap();
        writeln!(proto, r#"package app;"#).unwrap();
        writeln!(proto).unwrap();
        writeln!(proto, r#"import "google/protobuf/timestamp.proto";"#).unwrap();

        proto
    }

    fn write_protobuf_message(&self, message: &ProtobufMessage) -> String {
        let mut proto = String::new();

        writeln!(proto, "message {} {{", message.name).unwrap();

        for (i, field) in message.fields.iter().enumerate() {
            writeln!(proto, "  {}", self.write_protobuf_field(field, i)).unwrap();
        }

        writeln!(proto, "}}").unwrap();

        proto
    }

    fn write_protobuf_field(&self, field: &ProtobufField, index: usize) -> String {
        let mut proto = String::new();

        if !field.required {
            write!(proto, "optional ").unwrap();
        }

        write!(
            proto,
            "{ty} {name} = {num};",
            ty = field.ty,
            name = field.name,
            num = index + 1
        )
        .unwrap();

        proto
    }

    fn write_protobuf_service(&self, service: &ProtobufService) -> String {
        let mut proto = String::new();

        writeln!(proto, "service {} {{", service.name).unwrap();

        for method in &service.methods {
            write!(proto, "  {}", self.write_protobuf_method(method)).unwrap();
        }

        writeln!(proto, "}}").unwrap();

        proto
    }

    fn write_protobuf_method(&self, method: &ProtobufMethod) -> String {
        let mut proto = String::new();

        writeln!(
            proto,
            "rpc {name}({param}) returns ({returns});",
            name = method.name,
            param = method.param.name,
            returns = method.returns.name,
        )
        .unwrap();

        proto
    }
}
