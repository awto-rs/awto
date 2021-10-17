use std::{
    fmt::{self, Debug},
    str,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    Custom(ProtobufSchema),
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

impl fmt::Display for ProtobufType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Double => write!(f, "double"),
            Self::Float => write!(f, "float"),
            Self::Int32 => write!(f, "int32"),
            Self::Int64 => write!(f, "int64"),
            Self::Uint32 => write!(f, "uint32"),
            Self::Uint64 => write!(f, "uint64"),
            Self::Sint32 => write!(f, "sint32"),
            Self::Sint64 => write!(f, "sint64"),
            Self::Fixed32 => write!(f, "fixed32"),
            Self::Fixed64 => write!(f, "fixed64"),
            Self::Sfixed32 => write!(f, "sfixed32"),
            Self::Sfixed64 => write!(f, "sfixed64"),
            Self::Bool => write!(f, "bool"),
            Self::String => write!(f, "string"),
            Self::Bytes => write!(f, "bytes"),
            Self::Repeated(inner) => write!(f, "repeated {}", inner),
            Self::Timestamp => write!(f, "google.protobuf.Timestamp"),
            Self::Custom(inner) => write!(f, "{}", inner.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtobufField {
    pub name: String,
    pub ty: ProtobufType,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtobufMethod {
    pub name: String,
    pub param: ProtobufSchema,
    pub returns: ProtobufSchema,
}

pub trait IntoProtobufSchema {
    fn protobuf_schema() -> ProtobufSchema;
}

pub trait IntoProtobufService {
    fn protobuf_service() -> ProtobufService;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtobufSchema {
    pub name: String,
    pub fields: Vec<ProtobufField>,
    pub generated_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtobufService {
    pub name: String,
    pub methods: Vec<ProtobufMethod>,
    pub generated_code: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate as awto;
    use crate::prelude::*;

    #[derive(ProtobufModel)]
    pub struct Product {
        pub name: String,
        #[awto(default = 0)]
        pub price: i64,
        #[awto(max_len = 256)]
        pub description: Option<String>,
        pub comments: Vec<String>,
    }

    #[test]
    fn message_name() {
        assert_eq!(Product::protobuf_schema().name, "Product");
    }

    #[test]
    fn columns() {
        let fields = Product::protobuf_schema().fields;

        let expected = vec![
            ProtobufField {
                name: "name".to_string(),
                ty: ProtobufType::String,
                required: true,
            },
            ProtobufField {
                name: "price".to_string(),
                ty: ProtobufType::Int64,
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
