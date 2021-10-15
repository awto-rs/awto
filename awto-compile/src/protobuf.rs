use std::{env, fmt::Write};

use awto::AwtoApp;
use awto_schema::protobuf::{ProtobufSchema, ProtobufService};
use tokio::{fs, io::AsyncWriteExt};

#[cfg(not(feature = "blocking"))]
pub async fn compile_protobuf<A: AwtoApp>(app: A) -> Result<(), Box<dyn std::error::Error>> {
    compile_protobuf_async(app).await
}

#[cfg(feature = "blocking")]
pub fn compile_protobuf<A: AwtoApp>(app: A) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(compile_protobuf_async(app))
}

async fn compile_protobuf_async<A: AwtoApp>(_app: A) -> Result<(), Box<dyn std::error::Error>> {
    let app_config = A::app_config();
    let out_dir = env::var("OUT_DIR").unwrap();

    if !app_config.compile_protobuf {
        return Ok(());
    }

    let protobuf_schemas = A::protobuf_schemas();
    let protobuf_services = A::protobuf_services();

    let proto = compile_protobuf_items(protobuf_schemas, protobuf_services);
    let proto_path = format!("{}/schema.proto", out_dir);
    fs::write(&proto_path, proto + "\n").await?;

    tonic_build::configure().compile(&[&proto_path], &[&out_dir])?;

    let generated_code = compile_protobuf_generated_code(protobuf_schemas, protobuf_services);
    if !generated_code.is_empty() {
        let rs_path = format!("{}/schema.rs", out_dir);
        let mut schema_file = fs::OpenOptions::new().append(true).open(&rs_path).await?;

        schema_file.write(generated_code.as_bytes()).await?;
        schema_file.sync_all().await?;
    }

    Ok(())
}

fn compile_protobuf_items(
    messages: &[&dyn ProtobufSchema],
    services: &[&dyn ProtobufService],
) -> String {
    let mut proto = String::new();

    writeln!(
        proto,
        r#"syntax = "proto3";

package schema;

import "google/protobuf/timestamp.proto";
"#
    )
    .unwrap();

    let service_messages: Vec<_> = services
        .iter()
        .map(|service| {
            service
                .methods()
                .into_iter()
                .map(|method| vec![method.param, method.returns])
                .collect::<Vec<_>>()
        })
        .flatten()
        .flatten()
        .collect::<Vec<_>>();
    let mut all_message: Vec<&dyn ProtobufSchema> =
        Vec::with_capacity(messages.len() + service_messages.len());
    all_message.extend(messages);
    all_message.extend(service_messages.iter().map(|msg| msg.as_ref()));
    all_message =
        all_message
            .into_iter()
            .fold(Vec::new(), |mut acc, message: &dyn ProtobufSchema| {
                if !acc
                    .iter()
                    .any(|a| a.message_name() == message.message_name())
                {
                    acc.push(message)
                }
                acc
            });

    for message in all_message {
        let message_proto = generate_protobuf_message(message);
        writeln!(proto, "{}\n", message_proto).unwrap();
    }

    for service in services {
        let service_proto = generate_protobuf_service(*service);
        writeln!(proto, "{}\n", service_proto).unwrap();
    }

    proto.trim().to_string()
}

fn compile_protobuf_generated_code(
    messages: &[&dyn ProtobufSchema],
    services: &[&dyn ProtobufService],
) -> String {
    let mut generated_code = String::new();

    let service_messages: Vec<_> = services
        .iter()
        .map(|service| {
            service
                .methods()
                .into_iter()
                .map(|method| vec![method.param, method.returns])
                .collect::<Vec<_>>()
        })
        .flatten()
        .flatten()
        .collect::<Vec<_>>();
    let mut all_message: Vec<&dyn ProtobufSchema> =
        Vec::with_capacity(messages.len() + service_messages.len());
    all_message.extend(messages);
    all_message.extend(service_messages.iter().map(|msg| msg.as_ref()));
    all_message =
        all_message
            .into_iter()
            .fold(Vec::new(), |mut acc, message: &dyn ProtobufSchema| {
                if !acc
                    .iter()
                    .any(|a| a.message_name() == message.message_name())
                {
                    acc.push(message)
                }
                acc
            });

    for message in all_message {
        write!(generated_code, "{}", message.code()).unwrap();
    }

    for service in services {
        write!(generated_code, "{}", service.code()).unwrap();
    }

    generated_code.trim().to_string()
}

fn generate_protobuf_message(message: &dyn ProtobufSchema) -> String {
    let mut proto = String::new();

    writeln!(proto, "message {} {{", message.message_name()).unwrap();
    for (i, field) in message.fields().iter().enumerate() {
        let required = if field.required { "" } else { "optional " };
        writeln!(
            proto,
            "  {required}{ty} {name} = {num};",
            required = required,
            ty = field.ty.to_string(),
            name = field.name,
            num = i + 1
        )
        .unwrap();
    }
    write!(proto, "}}").unwrap();

    proto
}

fn generate_protobuf_service(service: &dyn ProtobufService) -> String {
    let mut proto = String::new();

    writeln!(proto, "service {} {{", service.service_name()).unwrap();
    for method in service.methods() {
        writeln!(
            proto,
            "  rpc {name}({param}) returns ({returns});",
            name = method.name,
            param = method.param.message_name(),
            returns = method.returns.message_name(),
        )
        .unwrap();
    }
    write!(proto, "}}").unwrap();

    proto
}

#[cfg(test)]
mod test {
    use super::*;
    use awto_schema::protobuf::IntoProtobufSchema;
    use awto_schema::*;
    use uuid::Uuid;

    #[derive(Model)]
    pub struct Product {
        pub name: String,
        #[awto(default = 0)]
        pub price: u64,
        #[awto(max_len = 256)]
        pub description: Option<String>,
    }

    #[derive(Model)]
    pub struct Variant {
        #[awto(references = (Product, "id"))]
        pub product_id: Uuid,
        pub name: String,
        pub price: u64,
    }

    #[test]
    fn messages() {
        let sql = compile_protobuf(
            &[&Product::protobuf_schema(), &Variant::protobuf_schema()],
            &[],
        );
        assert_eq!(
            sql,
            r#"syntax = "proto3";

package schema;

import "google/protobuf/timestamp.proto";

message Product {
  string id = 1;
  google.protobuf.Timestamp created_at = 2;
  google.protobuf.Timestamp updated_at = 3;
  string name = 4;
  uint64 price = 5;
  optional string description = 6;
}

message Variant {
  string id = 1;
  google.protobuf.Timestamp created_at = 2;
  google.protobuf.Timestamp updated_at = 3;
  string product_id = 4;
  string name = 5;
  uint64 price = 6;
}"#
        )
    }
}
