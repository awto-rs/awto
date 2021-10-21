use crate::database::DatabaseTable;
use crate::protobuf::ProtobufMessage;

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    /// An item which will be used to create database tables
    DatabaseTable(DatabaseTable),

    /// An item containing some fields of an existing DatabaseTable item, typically used for inserts or updates
    DatabaseSubTable(DatabaseTable),

    /// An item which will be used as a protobuf message
    ProtobufMessage(ProtobufMessage),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustField {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub name: String,
    pub roles: Vec<Role>,
    pub fields: Vec<RustField>,
}
