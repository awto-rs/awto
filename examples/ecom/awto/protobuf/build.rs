use awto::{schema::Schema, service::Service};
use awto_compile::protobuf::compile_protobuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_protobuf(
        schema::Schema::protobuf_schemas(),
        service::Service::protobuf_services(),
    )
}
