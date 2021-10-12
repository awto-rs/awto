use std::fmt::Write;

use awto_schema::database::DatabaseSchema;

pub fn compile_database(tables: &[&dyn DatabaseSchema]) -> String {
    let mut sql = String::new();

    for (i, table) in tables.iter().enumerate() {
        let create_table_sql = generate_table_create_sql(*table);
        writeln!(sql, "{}", create_table_sql).unwrap();
        if i < tables.len() - 1 {
            writeln!(sql).unwrap();
        }
    }

    sql
}

fn generate_table_create_sql(table: &dyn DatabaseSchema) -> String {
    let mut sql = String::new();

    writeln!(sql, "CREATE TABLE IF NOT EXISTS {} (", table.table_name()).unwrap();
    let columns = table.columns();
    for (i, column) in columns.iter().enumerate() {
        let null = if column.nullable { "NULL" } else { "NOT NULL" };
        let default = column
            .default
            .as_ref()
            .map(|default| format!(" DEFAULT {}", default))
            .unwrap_or_default();
        let constraint = column
            .constraint
            .as_ref()
            .map(|constraint| format!(" CHECK ({})", constraint))
            .unwrap_or_default();
        let primary_key = if column.primary_key {
            " PRIMARY KEY"
        } else {
            ""
        };
        let references = column
            .references
            .map(|(t, c)| format!(" REFERENCES {}({})", t, c))
            .unwrap_or_default();
        write!(
            sql,
            "  {name} {ty} {null}{default}{constraint}{primary_key}{references}",
            name = column.name,
            ty = column.ty,
            null = null,
            default = default,
            constraint = constraint,
            primary_key = primary_key,
            references = references,
        )
        .unwrap();

        if i < columns.len() - 1 {
            writeln!(sql, ",").unwrap();
        } else {
            writeln!(sql).unwrap();
        }
    }
    write!(sql, ");").unwrap();

    sql
}

#[cfg(test)]
mod test {
    use super::*;
    use awto_schema::database::IntoDatabaseSchema;
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
        #[awto(references = ("product", "id"))]
        pub product_id: Uuid,
        pub name: String,
        pub price: u64,
    }

    #[test]
    fn create_tables() {
        let sql = compile_database(&[&Product::database_schema(), &Variant::database_schema()]);
        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS product (
  id uuid NOT NULL DEFAULT uuid_generate_v4() PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW(),
  name varchar NOT NULL,
  price bigint NOT NULL DEFAULT 0,
  description varchar(256) NULL
);

CREATE TABLE IF NOT EXISTS variant (
  id uuid NOT NULL DEFAULT uuid_generate_v4() PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW(),
  product_id uuid NOT NULL REFERENCES product(id),
  name varchar NOT NULL,
  price bigint NOT NULL
);
"
        )
    }
}
