use std::fmt::Write;

use awto_schema::protobuf::ProtobufSchema;

pub fn compile_protobuf(messages: &[&dyn ProtobufSchema]) -> String {
    let mut proto = String::new();

    writeln!(
        proto,
        r#"syntax = "proto3";

package schema;

import "google/protobuf/timestamp.proto";
"#
    )
    .unwrap();

    for (i, message) in messages.iter().enumerate() {
        let message_proto = generate_protobuf_message(*message);
        writeln!(proto, "{}", message_proto).unwrap();
        if i < messages.len() - 1 {
            writeln!(proto).unwrap();
        }
    }

    proto
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
    fn create_tables() {
        let sql = compile_protobuf(&[&Product::protobuf_schema(), &Variant::protobuf_schema()]);
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
}
"#
        )
    }
}
