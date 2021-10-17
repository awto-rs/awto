use std::{fmt, str};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DatabaseType {
    SmallInt,
    Integer,
    BigInt,
    Numeric(Option<(u16, u16)>),
    Float,
    Double,
    Money,
    Text(Option<i32>),
    Binary,
    Timestamp,
    Timestamptz,
    Date,
    Time,
    Timetz,
    Bool,
    Uuid,
}

pub struct DatabaseTypeFromStrError;

impl str::FromStr for DatabaseType {
    type Err = DatabaseTypeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let database_type = match s {
            "smallint" | "int2" => Self::SmallInt,
            "integer" | "int" | "int4" => Self::Integer,
            "bigint" | "int8" => Self::BigInt,
            "numeric" | "decimal" => Self::Numeric(None),
            "real" | "float4" => Self::Float,
            "double precision" | "float8" => Self::Double,
            "money" => Self::Money,
            "character" | "char" | "character varying" | "charvar" => Self::Text(None),
            "bytea" => Self::Binary,
            "timestamp" => Self::Timestamp,
            "timestamp with time zone" | "timestamptz" => Self::Timestamptz,
            "date" => Self::Date,
            "time" => Self::Time,
            "time with time zone" | "timetz" => Self::Timetz,
            "boolean" | "bool" => Self::Bool,
            "uuid" => Self::Uuid,
            _ => return Err(DatabaseTypeFromStrError),
        };
        Ok(database_type)
    }
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SmallInt => write!(f, "smallint"),
            Self::Integer => write!(f, "integer"),
            Self::BigInt => write!(f, "bigint"),
            Self::Numeric(Some((p, s))) => write!(f, "numeric({}, {})", p, s),
            Self::Numeric(None) => write!(f, "numeric"),
            Self::Float => write!(f, "real"),
            Self::Double => write!(f, "double precision"),
            Self::Money => write!(f, "money"),
            Self::Text(Some(max)) => write!(f, "character varying({})", max),
            Self::Text(None) => write!(f, "character varying"),
            Self::Binary => write!(f, "bytea"),
            Self::Timestamp => write!(f, "timestamp"),
            Self::Timestamptz => write!(f, "timestamp with time zone"),
            Self::Date => write!(f, "date"),
            Self::Time => write!(f, "time"),
            Self::Timetz => write!(f, "time with time zone"),
            Self::Bool => write!(f, "boolean"),
            Self::Uuid => write!(f, "uuid"),
        }
    }
}

#[derive(Clone, Debug, PartialOrd)]
pub enum DatabaseDefault {
    Bool(bool),
    Float(i64),
    Int(u64),
    Raw(String),
    String(String),
}

impl PartialEq for DatabaseDefault {
    fn eq(&self, other: &Self) -> bool {
        match self {
            DatabaseDefault::Bool(v) => match other {
                DatabaseDefault::Bool(other) => v == other,
                _ => false,
            },
            DatabaseDefault::Float(v) => match other {
                DatabaseDefault::Float(other) => v == other,
                _ => false,
            },
            DatabaseDefault::Int(v) => match other {
                DatabaseDefault::Int(other) => v == other,
                _ => false,
            },
            DatabaseDefault::Raw(v) => match other {
                DatabaseDefault::Raw(other) => v.to_lowercase() == other.to_lowercase(),
                _ => false,
            },
            DatabaseDefault::String(v) => match other {
                DatabaseDefault::String(other) => v == other,
                _ => false,
            },
        }
    }
}

impl fmt::Display for DatabaseDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseDefault::Bool(val) => write!(f, "{}", val),
            DatabaseDefault::Float(val) => write!(f, "{}", val),
            DatabaseDefault::Int(val) => write!(f, "{}", val),
            DatabaseDefault::Raw(val) => write!(f, "{}", val),
            DatabaseDefault::String(val) => write!(f, "\"{}\"", val),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DatabaseColumn {
    pub name: String,
    pub ty: DatabaseType,
    pub nullable: bool,
    pub default: Option<DatabaseDefault>,
    pub unique: bool,
    pub constraint: Option<String>,
    pub primary_key: bool,
    pub references: Option<(String, String)>,
}

pub trait IntoDatabaseSchema {
    fn database_schema() -> DatabaseSchema;
}

pub struct DatabaseSchema {
    pub table_name: String,
    pub columns: Vec<DatabaseColumn>,
    pub generated_code: Option<String>,
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, FixedOffset};
    use uuid::Uuid;

    use super::*;
    use crate as awto;
    use crate::prelude::*;

    #[derive(Model)]
    pub struct Product {
        pub id: Uuid,
        pub created_at: DateTime<FixedOffset>,
        pub updated_at: DateTime<FixedOffset>,
        pub name: String,
        #[awto(default = 0)]
        pub price: i64,
        #[awto(max_len = 256)]
        pub description: Option<String>,
    }

    #[test]
    fn table_name() {
        assert_eq!(Product::database_schema().table_name, "product");
    }

    #[test]
    fn columns() {
        let columns = Product::database_schema().columns;
        let expected = vec![
            DatabaseColumn {
                name: "id".to_string(),
                ty: DatabaseType::Uuid,
                nullable: false,
                default: Some(DatabaseDefault::Raw("uuid_generate_v4()".to_string())),
                unique: false,
                constraint: None,
                primary_key: true,
                references: None,
            },
            DatabaseColumn {
                name: "created_at".to_string(),
                ty: DatabaseType::Timestamptz,
                nullable: false,
                default: Some(DatabaseDefault::Raw("NOW()".to_string())),
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "updated_at".to_string(),
                ty: DatabaseType::Timestamptz,
                nullable: false,
                default: Some(DatabaseDefault::Raw("NOW()".to_string())),
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "name".to_string(),
                ty: DatabaseType::Text(None),
                nullable: false,
                default: None,
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "price".to_string(),
                ty: DatabaseType::BigInt,
                nullable: false,
                default: Some(DatabaseDefault::Int(0)),
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "description".to_string(),
                ty: DatabaseType::Text(Some(256)),
                nullable: true,
                default: None,
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
        ];
        assert_eq!(columns, expected);
    }
}
