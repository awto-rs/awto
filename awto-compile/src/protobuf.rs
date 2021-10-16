use std::{env, fmt::Write};

#[cfg(feature = "async")]
use async_trait::async_trait;
use awto_schema::protobuf::{ProtobufField, ProtobufMethod, ProtobufSchema, ProtobufService};

#[cfg(feature = "async")]
#[async_trait]
pub trait AppProtobufCompile {
    async fn compile_protobuf() -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(feature = "async")]
#[async_trait]
impl<App> AppProtobufCompile for App
where
    App: AwtoApp,
{
    async fn compile_protobuf() -> Result<(), Box<dyn std::error::Error>> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        let app_config = A::app_config();
        let out_dir = env::var("OUT_DIR").unwrap();

        if !app_config.compile_protobuf {
            return Ok(());
        }

        let protobuf_schemas = A::protobuf_schemas();
        let protobuf_services = A::protobuf_services();
        let protobuf_compiler = ProtobufCompiler::new(protobuf_schemas, protobuf_services);

        let proto = protobuf_compiler.compile_file();
        let proto_path = format!("{}/schema.proto", out_dir);
        fs::write(&proto_path, proto + "\n").await?;

        tonic_build::configure().compile(&[&proto_path], &[&out_dir])?;

        let generated_code = protobuf_compiler.compile_generated_code();
        if !generated_code.is_empty() {
            let rs_path = format!("{}/schema.rs", out_dir);
            let mut schema_file = fs::OpenOptions::new().append(true).open(&rs_path).await?;

            schema_file.write(generated_code.as_bytes()).await?;
            schema_file.sync_all().await?;
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
pub fn compile_protobuf(
    schemas: Vec<ProtobufSchema>,
    services: Vec<ProtobufService>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    let app_config = A::app_config();
    let out_dir = env::var("OUT_DIR").unwrap();

    if !app_config.compile_protobuf {
        return Ok(());
    }

    let protobuf_compiler = ProtobufCompiler::new(schemas, services);

    let proto = protobuf_compiler.compile_file();
    let proto_path = format!("{}/schema.proto", out_dir);
    fs::write(&proto_path, proto + "\n").await?;

    tonic_build::configure().compile(&[&proto_path], &[&out_dir])?;

    let generated_code = protobuf_compiler.compile_generated_code();
    if !generated_code.is_empty() {
        let rs_path = format!("{}/schema.rs", out_dir);
        let mut schema_file = fs::OpenOptions::new().append(true).open(&rs_path).await?;

        schema_file.write(generated_code.as_bytes()).await?;
        schema_file.sync_all().await?;
    }

    Ok(())
}

#[cfg(not(feature = "async"))]
pub fn compile_protobuf(
    schemas: Vec<ProtobufSchema>,
    services: Vec<ProtobufService>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;

    let out_dir = env::var("OUT_DIR").unwrap();

    let protobuf_compiler = ProtobufCompiler::new(schemas, services);

    let proto = protobuf_compiler.compile_file();
    let proto_path = format!("{}/schema.proto", out_dir);
    fs::write(&proto_path, proto + "\n")?;

    tonic_build::configure().compile(&[&proto_path], &[&out_dir])?;

    let generated_code = protobuf_compiler.compile_generated_code();
    if !generated_code.is_empty() {
        let rs_path = format!("{}/schema.rs", out_dir);
        let mut schema_file = fs::OpenOptions::new().append(true).open(&rs_path)?;

        write!(schema_file, "{}", generated_code)?;
        schema_file.sync_all()?;
    }

    Ok(())
}

/// Compiles a protobuf schema from a slice of [`ProtobufSchema`]s and [`ProtobufService`]s.
///
/// # Examples
///
/// ```
/// # use awto_compile::protobuf::ProtobufCompiler;
/// # use awto_compile::tests_cfg::*;
/// let compiler = ProtobufCompiler::new(
///     &[&ProductProtobufSchema, &VariantProtobufSchema],
///     &[&ProductService],
/// );
/// let protobuf_file = compiler.compile_file();
///
/// assert_eq!(protobuf_file, r#"syntax = "proto3";
///
/// package schema;
///
/// import "google/protobuf/timestamp.proto";
///
/// message Product {
///   string id = 1;
///   google.protobuf.Timestamp created_at = 2;
///   google.protobuf.Timestamp updated_at = 3;
///   string name = 4;
///   uint64 price = 5;
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
/// message Variant {
///   string id = 1;
///   google.protobuf.Timestamp created_at = 2;
///   google.protobuf.Timestamp updated_at = 3;
///   string product_id = 4;
///   string name = 5;
///   uint64 price = 6;
/// }
///
/// service ProductService {
///   rpc FindProduct(ProductId) returns (ProductList);
/// }"#);
/// ```
pub struct ProtobufCompiler {
    messages: Vec<ProtobufSchema>,
    services: Vec<ProtobufService>,
}

impl ProtobufCompiler {
    /// Creates a new instance of [`ProtobufCompiler`].
    pub fn new(messages: Vec<ProtobufSchema>, services: Vec<ProtobufService>) -> ProtobufCompiler {
        ProtobufCompiler { messages, services }
    }

    /// Compiles a protobuf file.
    pub fn compile_file(&self) -> String {
        let mut proto = String::new();

        write!(proto, "{}", self.write_protobuf_header()).unwrap();
        writeln!(proto).unwrap();

        for message in self.all_distinct_messages() {
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

        for message in self.all_distinct_messages() {
            if let Some(generated_code) = &message.generated_code {
                write!(code, "{}", generated_code).unwrap();
            }
        }

        for service in &self.services {
            if let Some(generated_code) = &service.generated_code {
                write!(code, "{}", generated_code).unwrap();
            }
        }

        code.trim().to_string()
    }

    fn all_distinct_messages(&self) -> Vec<&ProtobufSchema> {
        let mut all_messages = self.services.iter().fold(Vec::new(), |mut acc, service| {
            for method in &service.methods {
                acc.push(&method.param);
                acc.push(&method.returns);
            }

            acc
        });
        all_messages.extend(&self.messages);

        all_messages.sort_by_key(|message| message.name.as_str());
        all_messages.dedup_by_key(|message| message.name.as_str());

        all_messages
    }

    fn write_protobuf_header(&self) -> String {
        let mut proto = String::new();

        writeln!(proto, r#"syntax = "proto3";"#).unwrap();
        writeln!(proto).unwrap();
        writeln!(proto, r#"package schema;"#).unwrap();
        writeln!(proto).unwrap();
        writeln!(proto, r#"import "google/protobuf/timestamp.proto";"#).unwrap();

        proto
    }

    fn write_protobuf_message(&self, message: &ProtobufSchema) -> String {
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
