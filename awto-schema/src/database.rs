use std::fmt;

pub const DEFAULT_DATABASE_COLUMNS: [DatabaseColumn; 3] = [
    DatabaseColumn {
        name: "id",
        ty: "uuid",
        nullable: false,
        default: Some(DatabaseDefault::Raw("uuid_generate_v4()")),
        unique: false,
        constraint: None,
        primary_key: true,
        references: None,
    },
    DatabaseColumn {
        name: "created_at",
        ty: "timestamptz",
        nullable: false,
        default: Some(DatabaseDefault::Raw("NOW()")),
        unique: false,
        constraint: None,
        primary_key: false,
        references: None,
    },
    DatabaseColumn {
        name: "updated_at",
        ty: "timestamptz",
        nullable: false,
        default: Some(DatabaseDefault::Raw("NOW()")),
        unique: false,
        constraint: None,
        primary_key: false,
        references: None,
    },
];

#[derive(Clone, Debug, PartialEq)]
pub enum DatabaseDefault {
    Bool(bool),
    Float(i64),
    Int(u64),
    Raw(&'static str),
    String(&'static str),
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

#[derive(Clone, Debug, PartialEq)]
pub struct DatabaseColumn {
    pub name: &'static str,
    pub ty: &'static str,
    pub nullable: bool,
    pub default: Option<DatabaseDefault>,
    pub unique: bool,
    pub constraint: Option<&'static str>,
    pub primary_key: bool,
    pub references: Option<(&'static str, &'static str)>,
}

pub trait IntoDatabaseSchema {
    type Schema: DatabaseSchema + Default;

    fn database_schema() -> Self::Schema {
        Self::Schema::default()
    }
}

pub trait DatabaseSchema {
    fn table_name(&self) -> &'static str;

    fn columns(&self) -> Vec<DatabaseColumn>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate as awto_schema;
    use crate::*;

    #[derive(Model)]
    pub struct Product {
        pub name: String,
        #[awto(default = 0)]
        pub price: u64,
        #[awto(max_len = 256)]
        pub description: Option<String>,
    }

    #[test]
    fn table_name() {
        assert_eq!(Product::database_schema().table_name(), "product");
    }

    #[test]
    fn columns() {
        let columns = Product::database_schema().columns();
        let expected = vec![
            DatabaseColumn {
                name: "id",
                ty: "uuid",
                nullable: false,
                default: Some(DatabaseDefault::Raw("uuid_generate_v4()")),
                unique: false,
                constraint: None,
                primary_key: true,
                references: None,
            },
            DatabaseColumn {
                name: "created_at",
                ty: "timestamptz",
                nullable: false,
                default: Some(DatabaseDefault::Raw("NOW()")),
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "updated_at",
                ty: "timestamptz",
                nullable: false,
                default: Some(DatabaseDefault::Raw("NOW()")),
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "name",
                ty: "varchar",
                nullable: false,
                default: None,
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "price",
                ty: "bigint",
                nullable: false,
                default: Some(DatabaseDefault::Int(0)),
                unique: false,
                constraint: None,
                primary_key: false,
                references: None,
            },
            DatabaseColumn {
                name: "description",
                ty: "varchar(256)",
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
