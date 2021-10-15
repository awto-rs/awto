use std::{fmt::Debug, str, string};

use dyn_clone::DynClone;

#[derive(Clone, Debug)]
pub enum ProtobufType {
    Double,
    Float,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    String,
    Bytes,
    Repeated(Box<ProtobufType>),
    Timestamp,
    Custom(Box<dyn ProtobufSchema>),
}

pub struct ProtobufTypeFromStrError;

impl str::FromStr for ProtobufType {
    type Err = ProtobufTypeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(inner) = s.strip_prefix("repeated ") {
            return Ok(Self::Repeated(Box::new(inner.parse()?)));
        }
        let protobuf_type = match s {
            "double" => Self::Double,
            "float" => Self::Float,
            "int32" => Self::Int32,
            "int64" => Self::Int64,
            "uint32" => Self::Uint32,
            "uint64" => Self::Uint64,
            "sint32" => Self::Sint32,
            "sint64" => Self::Sint64,
            "fixed32" => Self::Fixed32,
            "fixed64" => Self::Fixed64,
            "sfixed32" => Self::Sfixed32,
            "sfixed64" => Self::Sfixed64,
            "bool" => Self::Bool,
            "String" => Self::String,
            "bytes" => Self::Bytes,
            "google.protobuf.Timestamp" => Self::Timestamp,
            _ => return Err(ProtobufTypeFromStrError),
        };
        Ok(protobuf_type)
    }
}

impl string::ToString for ProtobufType {
    fn to_string(&self) -> String {
        match self {
            Self::Double => "double".to_string(),
            Self::Float => "float".to_string(),
            Self::Int32 => "int32".to_string(),
            Self::Int64 => "int64".to_string(),
            Self::Uint32 => "uint32".to_string(),
            Self::Uint64 => "uint64".to_string(),
            Self::Sint32 => "sint32".to_string(),
            Self::Sint64 => "sint64".to_string(),
            Self::Fixed32 => "fixed32".to_string(),
            Self::Fixed64 => "fixed64".to_string(),
            Self::Sfixed32 => "sfixed32".to_string(),
            Self::Sfixed64 => "sfixed64".to_string(),
            Self::Bool => "bool".to_string(),
            Self::String => "string".to_string(),
            Self::Bytes => "bytes".to_string(),
            Self::Repeated(inner) => format!("repeated {}", inner.to_string()),
            Self::Timestamp => "google.protobuf.Timestamp".to_string(),
            Self::Custom(inner) => inner.message_name().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ProtobufField {
    pub name: String,
    pub ty: ProtobufType,
    pub required: bool,
}

impl PartialEq for ProtobufField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.ty.to_string() == other.ty.to_string()
            && self.required == other.required
    }
}

pub struct ProtobufMethod {
    pub name: String,
    pub param: Box<dyn ProtobufSchema>,
    pub returns: Box<dyn ProtobufSchema>,
}

pub trait IntoProtobufSchema {
    type Schema: ProtobufSchema + Default;

    fn protobuf_schema() -> Self::Schema {
        Self::Schema::default()
    }
}

pub trait ProtobufSchema: ProtobufGeneratedCode + DynClone {
    fn message_name(&self) -> &'static str;

    fn fields(&self) -> Vec<ProtobufField>;
}

dyn_clone::clone_trait_object!(ProtobufSchema);

impl Debug for dyn ProtobufSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message_name())
    }
}

impl PartialEq for dyn ProtobufSchema {
    fn eq(&self, other: &Self) -> bool {
        self.message_name() == other.message_name()
    }
}

pub trait ProtobufService: ProtobufGeneratedCode {
    fn service_name(&self) -> &'static str;

    fn methods(&self) -> Vec<ProtobufMethod>;
}

pub trait ProtobufGeneratedCode {
    fn code(&self) -> &'static str;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate as awto_schema;
    use crate::*;

    #[derive(ProtobufModel)]
    pub struct Product {
        pub name: String,
        #[awto(default = 0)]
        pub price: u64,
        #[awto(max_len = 256)]
        pub description: Option<String>,
        pub comments: Vec<String>,
    }

    #[test]
    fn message_name() {
        assert_eq!(Product::protobuf_schema().message_name(), "Product");
    }

    #[test]
    fn columns() {
        let fields = Product::protobuf_schema().fields();

        let expected = vec![
            ProtobufField {
                name: "name".to_string(),
                ty: ProtobufType::String,
                required: true,
            },
            ProtobufField {
                name: "price".to_string(),
                ty: ProtobufType::Uint64,
                required: true,
            },
            ProtobufField {
                name: "description".to_string(),
                ty: ProtobufType::String,
                required: false,
            },
            ProtobufField {
                name: "comments".to_string(),
                ty: ProtobufType::Repeated(Box::new(ProtobufType::String)),
                required: true,
            },
        ];
        assert_eq!(fields, expected);
    }
}
