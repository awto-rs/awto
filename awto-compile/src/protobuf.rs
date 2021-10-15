use std::{env, fmt::Write};

use awto::AwtoApp;
use awto_schema::protobuf::{ProtobufMethod, ProtobufSchema, ProtobufService};

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
pub struct ProtobufCompiler<'a> {
    messages: &'a [&'a dyn ProtobufSchema],
    services: &'a [&'a dyn ProtobufService],
}

enum AggregatedMessage<'a> {
    Borrowed(&'a dyn ProtobufSchema),
    Owned(Box<dyn ProtobufSchema>),
}

impl<'a> AggregatedMessage<'a> {
    fn as_ref(&'a self) -> &'a dyn ProtobufSchema {
        match self {
            AggregatedMessage::Borrowed(val) => *val,
            AggregatedMessage::Owned(val) => val.as_ref(),
        }
    }
}

impl<'a> ProtobufCompiler<'a> {
    /// Creates a new instance of [`ProtobufCompiler`].
    pub fn new(
        messages: &'a [&'a dyn ProtobufSchema],
        services: &'a [&'a dyn ProtobufService],
    ) -> ProtobufCompiler<'a> {
        ProtobufCompiler { messages, services }
    }

    /// Compiles a protobuf file.
    pub fn compile_file(&self) -> String {
        let mut proto = String::new();

        write!(proto, "{}", self.write_protobuf_header()).unwrap();
        writeln!(proto).unwrap();

        for message in self.all_distinct_messages() {
            writeln!(proto, "{}", self.write_protobuf_message(message.as_ref())).unwrap();
        }

        for service in self.services {
            writeln!(proto, "{}", self.write_protobuf_service(*service)).unwrap();
        }

        proto.trim().to_string()
    }

    /// Compiles generated Rust code from schemas and services.
    pub fn compile_generated_code(&self) -> String {
        let mut code = String::new();

        for message in self.all_distinct_messages() {
            write!(code, "{}", message.as_ref().code()).unwrap();
        }

        for service in self.services {
            write!(code, "{}", service.code()).unwrap();
        }

        code.trim().to_string()
    }

    fn all_distinct_messages(&self) -> Vec<AggregatedMessage<'a>> {
        let mut all_messages = self.services.iter().fold(Vec::new(), |mut acc, service| {
            for method in (*service).methods() {
                acc.push(AggregatedMessage::Owned(method.param));
                acc.push(AggregatedMessage::Owned(method.returns));
            }

            acc
        });
        all_messages.extend(
            self.messages
                .iter()
                .map(|message| AggregatedMessage::Borrowed(*message)),
        );

        all_messages.sort_by_key(|message| message.as_ref().message_name());
        all_messages.dedup_by_key(|message| message.as_ref().message_name());

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

    fn write_protobuf_message(&self, message: &dyn ProtobufSchema) -> String {
        let mut proto = String::new();

        writeln!(proto, "message {} {{", message.message_name()).unwrap();

        for (i, field) in message.fields().iter().enumerate() {
            write!(proto, "  ").unwrap();

            if !field.required {
                write!(proto, "optional ").unwrap();
            }

            writeln!(
                proto,
                "{ty} {name} = {num};",
                ty = field.ty,
                name = field.name,
                num = i + 1
            )
            .unwrap();
        }

        writeln!(proto, "}}").unwrap();

        proto
    }

    fn write_protobuf_service(&self, service: &dyn ProtobufService) -> String {
        let mut proto = String::new();

        writeln!(proto, "service {} {{", service.service_name()).unwrap();

        service.methods().iter().for_each(|method| {
            write!(proto, "{}", self.write_protobuf_method(method)).unwrap();
        });

        writeln!(proto, "}}").unwrap();

        proto
    }

    fn write_protobuf_method(&self, method: &ProtobufMethod) -> String {
        let mut proto = String::new();

        writeln!(
            proto,
            "  rpc {name}({param}) returns ({returns});",
            name = method.name,
            param = method.param.message_name(),
            returns = method.returns.message_name(),
        )
        .unwrap();

        proto
    }
}

#[cfg(feature = "async")]
pub async fn compile_protobuf<A: AwtoApp>(app: A) -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(not(feature = "async"))]
pub fn compile_protobuf<A: AwtoApp>(_app: A) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;

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
