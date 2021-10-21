use awto::service::Service;
use awto_compile::protobuf::compile_protobuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_protobuf(
        schema::MODELS.to_vec(),
        service::Service::protobuf_services(),
    )
}
