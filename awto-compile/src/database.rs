use std::fmt::Write;

use awto_schema::database::{DatabaseColumn, DatabaseDefault, DatabaseSchema, DatabaseType};
use sqlx::PgPool;

use crate::error::Error;

pub async fn compile_database(
    pool: &PgPool,
    tables: &[&dyn DatabaseSchema],
) -> Result<String, Error> {
    let mut sql = String::new();

    for (i, table) in tables.iter().enumerate() {
        let db_columns = get_table(pool, "public", table.table_name()).await?;

        match db_columns {
            Some(db_columns) => {
                let schema_columns = table.columns();
                writeln!(
                    sql,
                    "{}",
                    generate_columns_sync_sql(table.table_name(), &schema_columns, &db_columns)
                )
                .unwrap();
            }
            None => {
                writeln!(sql, "{}", generate_table_create_sql(*table)).unwrap();
            }
        }

        if i < tables.len() - 1 {
            writeln!(sql).unwrap();
        }
    }

    Ok(sql.trim().to_string())
}

async fn get_table(
    pool: &PgPool,
    schema: &str,
    name: &str,
) -> Result<Option<Vec<DatabaseColumn>>, Error> {
    #[derive(Debug, sqlx::FromRow)]
    struct ColumnsQuery {
        column_name: String,
        column_default: Option<String>,
        is_nullable: String,
        data_type: String,
        character_maximum_length: Option<i32>,
        is_primary_key: bool,
        is_unique: bool,
        // has_constraint: bool,
        reference: Option<String>,
    }

    let raw_columns: Vec<ColumnsQuery> = sqlx::query_as(
        "
        SELECT column_name, column_default, is_nullable, data_type, character_maximum_length,
        (
            SELECT
                COUNT(*) > 0
            FROM information_schema.table_constraints tco
            JOIN information_schema.key_column_usage kcu 
            ON kcu.constraint_name = tco.constraint_name
            AND kcu.constraint_schema = tco.constraint_schema
            AND kcu.constraint_name = tco.constraint_name
            WHERE
                tco.constraint_type = 'PRIMARY KEY' AND
                kcu.table_schema = $1 AND
                kcu.table_name = $2 AND
                kcu.column_name = information_schema.columns.column_name
        ) as is_primary_key,
        (
            SELECT
                COUNT(*) > 0
            FROM information_schema.table_constraints tco
            JOIN information_schema.key_column_usage kcu 
            ON kcu.constraint_name = tco.constraint_name
            AND kcu.constraint_schema = tco.constraint_schema
            AND kcu.constraint_name = tco.constraint_name
            WHERE
                tco.constraint_type = 'UNIQUE' AND
                kcu.table_schema = $1 AND
                kcu.table_name = $2 AND
                kcu.column_name = information_schema.columns.column_name
        ) as is_unique,
        (
            SELECT CONCAT(
                rel_tco.table_name,
                ':',
                (
                    SELECT u.column_name
                    FROM information_schema.constraint_column_usage u
                    INNER JOIN information_schema.referential_constraints fk
                    ON
                        u.constraint_catalog = fk.unique_constraint_catalog AND
                        u.constraint_schema = fk.unique_constraint_schema AND
                        u.constraint_name = fk.unique_constraint_name
                    INNER JOIN information_schema.key_column_usage r
                    ON
                        r.constraint_catalog = fk.constraint_catalog AND
                        r.constraint_schema = fk.constraint_schema AND
                        r.constraint_name = fk.constraint_name
                    WHERE
                        fk.constraint_name = kcu.constraint_name AND
                        u.table_schema = kcu.table_schema AND
                        u.table_name = rel_tco.table_name
                )
            )
            FROM information_schema.table_constraints tco
            JOIN information_schema.key_column_usage kcu
            ON
                tco.constraint_schema = kcu.constraint_schema AND
                tco.constraint_name = kcu.constraint_name
            JOIN information_schema.referential_constraints rco
            ON
                tco.constraint_schema = rco.constraint_schema AND
                tco.constraint_name = rco.constraint_name
            JOIN information_schema.table_constraints rel_tco
            ON
                rco.unique_constraint_schema = rel_tco.constraint_schema AND
                rco.unique_constraint_name = rel_tco.constraint_name
            WHERE
                tco.constraint_type = 'FOREIGN KEY' AND
                kcu.table_name = $2 AND
                kcu.column_name = information_schema.columns.column_name
            GROUP BY
                kcu.table_schema,
                kcu.table_name,
                rel_tco.table_name,
                rel_tco.table_schema,
                kcu.constraint_name
            ORDER BY
                kcu.table_schema,
                kcu.table_name
        ) as reference
        FROM information_schema.columns
        WHERE table_schema = $1
        AND table_name = $2;
        ",
    )
    .bind(schema)
    .bind(name)
    .fetch_all(pool)
    .await
    .map_err(Error::Sqlx)?;

    if raw_columns.is_empty() {
        return Ok(None);
    }

    let columns: Vec<DatabaseColumn> = raw_columns
        .into_iter()
        .map(|col| {
            let column_name = col.column_name;
            let character_maximum_length = col.character_maximum_length;

            Ok(DatabaseColumn {
                name: column_name.clone(),
                ty: col
                    .data_type
                    .parse::<DatabaseType>()
                    .map(|database_type| {
                        if let Some(max_len) = character_maximum_length {
                            if matches!(database_type, DatabaseType::Text(None)) {
                                return DatabaseType::Text(Some(max_len));
                            }
                        }

                        database_type
                    })
                    .map_err(|_| Error::UnsupportedType(name.to_string(), column_name))?,
                nullable: col.is_nullable == "YES",
                default: col.column_default.map(|def| {
                    if def.starts_with('\'') {
                        let s = def
                            .strip_prefix('\'')
                            .unwrap()
                            .splitn(2, '\'')
                            .next()
                            .unwrap()
                            .to_string();
                        DatabaseDefault::String(s)
                    } else if def == "true" {
                        DatabaseDefault::Bool(true)
                    } else if def == "false" {
                        DatabaseDefault::Bool(false)
                    } else if let Ok(num) = def.parse::<u64>() {
                        DatabaseDefault::Int(num)
                    } else if let Ok(num) = def.parse::<i64>() {
                        DatabaseDefault::Float(num)
                    } else {
                        DatabaseDefault::Raw(def)
                    }
                }),
                unique: col.is_unique,
                // constraint: if col.has_constraint {
                //     Some(String::new())
                // } else {
                //     None
                // },
                constraint: None,
                primary_key: col.is_primary_key,
                references: if let Some(references) = col.reference {
                    let mut parts = references.splitn(2, ':');
                    if let Some(references_table) = parts.next() {
                        parts.next().map(|references_column| {
                            (references_table.to_string(), references_column.to_string())
                        })
                    } else {
                        None
                    }
                } else {
                    None
                },
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(Some(columns))
}

fn generate_table_create_sql(table: &dyn DatabaseSchema) -> String {
    let mut sql = String::new();

    writeln!(sql, "CREATE TABLE IF NOT EXISTS {} (", table.table_name()).unwrap();
    let columns = table.columns();
    for (i, column) in columns.iter().enumerate() {
        write!(sql, "  {}", generate_table_create_column_sql(column)).unwrap();

        if i < columns.len() - 1 {
            writeln!(sql, ",").unwrap();
        } else {
            writeln!(sql).unwrap();
        }
    }
    write!(sql, ");").unwrap();

    sql
}

fn generate_table_create_column_sql(column: &DatabaseColumn) -> String {
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
        .as_ref()
        .map(|(t, c)| format!(" REFERENCES {}({})", t, c))
        .unwrap_or_default();

    format!(
        "{name} {ty} {null}{default}{constraint}{primary_key}{references}",
        name = column.name,
        ty = column.ty.to_string(),
        null = null,
        default = default,
        constraint = constraint,
        primary_key = primary_key,
        references = references,
    )
}

fn generate_columns_sync_sql(
    table: &str,
    schema_columns: &[DatabaseColumn],
    db_columns: &[DatabaseColumn],
) -> String {
    let mut sql = String::new();

    for schema_col in schema_columns {
        let db_col = match db_columns
            .iter()
            .find(|db_col| db_col.name == schema_col.name)
        {
            Some(db_col) => db_col,
            None => {
                // Column does not exist in DB
                writeln!(
                    sql,
                    "ALTER TABLE {} ADD COLUMN {};",
                    table,
                    generate_table_create_column_sql(schema_col)
                )
                .unwrap();
                continue;
            }
        };

        // Check for type mismatch
        // println!("{}, {}", schema_col.ty, db_col.ty);
        if schema_col.ty != db_col.ty {
            writeln!(
                sql,
                "ALTER TABLE {table} ALTER COLUMN {column} TYPE {ty} USING {column}::{ty};",
                table = table,
                column = schema_col.name,
                ty = schema_col.ty.to_string(),
            )
            .unwrap();
        }

        // Check for nullable mismatch
        if schema_col.nullable != db_col.nullable {
            if db_col.nullable {
                writeln!(
                    sql,
                    "ALTER TABLE {table} ALTER COLUMN {column} SET NOT NULL;",
                    table = table,
                    column = schema_col.name
                )
                .unwrap();
            } else {
                writeln!(
                    sql,
                    "ALTER TABLE {table} ALTER COLUMN {column} DROP NOT NULL;",
                    table = table,
                    column = schema_col.name
                )
                .unwrap();
            }
        }

        // Check for default mismatch
        if schema_col.default != db_col.default {
            if let Some(default) = &schema_col.default {
                writeln!(
                    sql,
                    "ALTER TABLE {table} ALTER COLUMN {column} SET DEFAULT {default};",
                    table = table,
                    column = schema_col.name,
                    default = default
                )
                .unwrap();
            } else {
                writeln!(
                    sql,
                    "ALTER TABLE {table} ALTER COLUMN {column} DROP DEFAULT;",
                    table = table,
                    column = schema_col.name
                )
                .unwrap();
            }
        }

        // Check for unique mismatch
        if schema_col.unique != db_col.unique {
            if db_col.unique {
                writeln!(
                    sql,
                    "ALTER TABLE {table} DROP CONSTRAINT {table}_{column}_key;",
                    table = table,
                    column = schema_col.name
                )
                .unwrap();
            } else {
                writeln!(
                    sql,
                    "ALTER TABLE {table} ADD CONSTRAINT {table}_{column}_key UNIQUE ({column});",
                    table = table,
                    column = schema_col.name
                )
                .unwrap();
            }
        }

        // Check for constraint mismatch
        // if schema_col.constraint.is_some()
        //     || (schema_col.constraint.is_none() && db_col.constraint.is_some())
        // {
        //     // TODO: Currently it only checks if a constraint exists or not, but not the actual constraint code itself
        //     writeln!(
        //         sql,
        //         "ALTER TABLE {table} DROP CONSTRAINT {table}_{column}_check;",
        //         table = table,
        //         column = schema_col.name
        //     )
        //     .unwrap();
        //     if let Some(constraint) = &schema_col.constraint {
        //         writeln!(
        //             sql,
        //             "ALTER TABLE {table} ADD CONSTRAINT {table}_{column}_check CHECK ({constraint});",
        //             table = table,
        //             column = schema_col.name,
        //             constraint = constraint
        //         )
        //         .unwrap();
        //     }
        // }

        // Check for references mismatch
        if schema_col.references != db_col.references {
            if let Some(references) = &schema_col.references {
                if db_col.references.is_some() {
                    writeln!(
                        sql,
                        "ALTER TABLE {table} DROP CONSTRAINT {table}_{column}_fkey;",
                        table = table,
                        column = schema_col.name
                    )
                    .unwrap();
                }
                writeln!(
                    sql,
                    "ALTER TABLE {table} ADD CONSTRAINT {table}_{column}_fkey FOREIGN KEY ({column}) REFERENCES {reference_table} ({reference_column});",
                    table = table,
                    column = schema_col.name,
                    reference_table = references.0,
                    reference_column = references.1,
                )
                .unwrap();
            } else {
                writeln!(
                    sql,
                    "ALTER TABLE {table} DROP CONSTRAINT {table}_{column}_fkey;",
                    table = table,
                    column = schema_col.name
                )
                .unwrap();
            }
        }
    }

    // Delete columns that exist in db but don't exist in schema
    db_columns
        .iter()
        .filter(|db_col| {
            schema_columns
                .iter()
                .all(|schema_col| schema_col.name != db_col.name)
        })
        .for_each(|db_col| {
            writeln!(
                sql,
                "ALTER TABLE {table} DROP COLUMN {column};",
                table = table,
                column = db_col.name
            )
            .unwrap();
        });

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
        #[awto(max_len = 120)]
        pub description: Option<String>,
    }

    #[derive(Model)]
    pub struct Variant {
        #[awto(references = (Product, "id"))]
        pub product_id: Uuid,
        pub name: String,
        pub price: u64,
    }

    #[tokio::test]
    async fn create_tables() {
        let pool = PgPool::connect("postgres://ari@0.0.0.0:5432/product")
            .await
            .unwrap();

        let _sql = compile_database(
            &pool,
            &[&Product::database_schema(), &Variant::database_schema()],
        )
        .await
        .unwrap();

        //         assert_eq!(
        //             sql,
        //             "CREATE TABLE IF NOT EXISTS product (
        //   id uuid NOT NULL DEFAULT uuid_generate_v4() PRIMARY KEY,
        //   created_at timestamptz NOT NULL DEFAULT NOW(),
        //   updated_at timestamptz NOT NULL DEFAULT NOW(),
        //   name varchar NOT NULL,
        //   price bigint NOT NULL DEFAULT 0,
        //   description varchar(256) NULL
        // );

        // CREATE TABLE IF NOT EXISTS variant (
        //   id uuid NOT NULL DEFAULT uuid_generate_v4() PRIMARY KEY,
        //   created_at timestamptz NOT NULL DEFAULT NOW(),
        //   updated_at timestamptz NOT NULL DEFAULT NOW(),
        //   product_id uuid NOT NULL REFERENCES product(id),
        //   name varchar NOT NULL,
        //   price bigint NOT NULL
        // );
        // "
        //         )
    }
}
