use std::{borrow::Cow, env, fmt::Write};

use awto::{
    database::{DatabaseColumn, DatabaseDefault, DatabaseTable, DatabaseType},
    schema::{Model, Role},
};
use proc_macro2::Literal;
use quote::{format_ident, quote};
use sqlx::{Executor, PgPool};
use tokio_stream::StreamExt;

use crate::{
    error::Error,
    util::{is_ty_option, is_ty_vec, strip_ty_option},
};

const COMPILED_RUST_FILE: &str = "app.rs";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompileDatabaseResult {
    pub queries_executed: usize,
    pub rows_affected: u64,
}

#[cfg(feature = "async")]
pub async fn compile_database(
    uri: &str,
    models: Vec<Model>,
) -> Result<CompileDatabaseResult, Box<dyn std::error::Error>> {
    use tokio::fs;

    let out_dir = env::var("OUT_DIR").unwrap();
    let pool = PgPool::connect(uri).await?;
    let compiler = DatabaseCompiler::from_pool(&pool, models);

    let generated_code = compiler.compile_generated_code();
    if !generated_code.is_empty() {
        let rs_path = format!("{}/{}", out_dir, COMPILED_RUST_FILE);
        fs::write(rs_path, generated_code).await?;
    }

    let sql = compiler.compile().await?;
    if !sql.is_empty() {
        let results = pool
            .execute_many(sql.as_str())
            .collect::<Result<Vec<_>, _>>()
            .await?;
        let queries_executed = results.len();
        let rows_affected = results
            .iter()
            .fold(0, |acc, result| result.rows_affected() + acc);

        Ok(CompileDatabaseResult {
            queries_executed,
            rows_affected,
        })
    } else {
        Ok(CompileDatabaseResult::default())
    }
}

#[cfg(not(feature = "async"))]
pub async fn compile_database(
    uri: &str,
    models: Vec<Model>,
) -> Result<CompileDatabaseResult, Box<dyn std::error::Error>> {
    use std::fs;

    let out_dir = env::var("OUT_DIR").unwrap();
    let pool = PgPool::connect(uri).await?;
    let compiler = DatabaseCompiler::from_pool(&pool, models);

    let generated_code = compiler.compile_generated_code();
    if !generated_code.is_empty() {
        let rs_path = format!("{}/{}", out_dir, COMPILED_RUST_FILE);
        fs::write(rs_path, generated_code)?;
    }

    let sql = compiler.compile().await?;
    if !sql.is_empty() {
        let results = pool
            .execute_many(sql.as_str())
            .collect::<Result<Vec<_>, _>>()
            .await?;
        let queries_executed = results.len();
        let rows_affected = results
            .iter()
            .fold(0, |acc, result| result.rows_affected() + acc);

        Ok(CompileDatabaseResult {
            queries_executed,
            rows_affected,
        })
    } else {
        Ok(CompileDatabaseResult::default())
    }
}

pub struct DatabaseCompiler<'pool> {
    pool: Cow<'pool, PgPool>,
    models: Vec<Model>,
}

impl<'pool> DatabaseCompiler<'pool> {
    pub async fn connect(
        uri: &str,
        models: Vec<Model>,
    ) -> Result<DatabaseCompiler<'_>, sqlx::Error> {
        let pool = sqlx::PgPool::connect(uri).await?;

        Ok(DatabaseCompiler {
            pool: Cow::Owned(pool),
            models,
        })
    }

    pub fn from_pool(pool: &'pool PgPool, models: Vec<Model>) -> DatabaseCompiler<'pool> {
        DatabaseCompiler {
            pool: Cow::Borrowed(pool),
            models,
        }
    }

    pub async fn compile(&self) -> Result<String, Error> {
        let mut sql = String::new();

        for (_, table) in self.database_tables() {
            let db_columns = self.fetch_table(table).await?;

            match db_columns {
                Some(db_columns) => {
                    writeln!(sql, "{}", self.write_sync_sql(table, &db_columns).await).unwrap();
                }
                None => {
                    writeln!(sql, "{}", self.write_table_create_sql(table)).unwrap();
                }
            }
        }

        Ok(sql.trim().to_string())
    }

    /// Compiles generated Rust code from schemas and services.
    pub fn compile_generated_code(&self) -> String {
        let mut code = String::new();

        for (model, table) in self.database_tables() {
            let ident = format_ident!("{}", model.name);
            let db_module_ident = format_ident!("{}", table.name);

            let mut from_schema_fields = Vec::new();
            let mut from_db_fields = Vec::new();

            for field in &model.fields {
                let field_ident = format_ident!("{}", field.name);

                let ty = strip_ty_option(&field.ty);

                if is_ty_vec(ty) {
                    from_schema_fields.push(
                        quote!(#field_ident: val.#field_ident.into_iter().map(|v| v.into()).collect()),
                    );
                    from_db_fields.push(
                        quote!(#field_ident: val.#field_ident.into_iter().map(|v| v.into()).collect()),
                    );
                } else {
                    from_schema_fields.push(quote!(#field_ident: val.#field_ident.into()));
                    from_db_fields.push(quote!(#field_ident: val.#field_ident.into()));
                }
            }

            let expanded = quote!(
                impl ::std::convert::From<crate::#db_module_ident::Model> for ::schema::#ident {
                    #[allow(unused_variables)]
                    fn from(val: crate::#db_module_ident::Model) -> Self {
                        Self {
                            #( #from_schema_fields, )*
                        }
                    }
                }

                impl ::std::convert::From<::schema::#ident> for crate::#db_module_ident::Model {
                    #[allow(unused_variables)]
                    fn from(val: ::schema::#ident) -> Self {
                        Self {
                            #( #from_db_fields, )*
                        }
                    }
                }
            );

            write!(code, "{}", expanded.to_string()).unwrap();
        }

   
        for (model, table) in self.database_sub_tables() {
            let ident = format_ident!("{}", model.name);
            let db_module_ident = format_ident!("{}", table.name);

            let active_values = model.fields.iter().map(|field| {
                let field_ident = format_ident!("{}", field.name);
                
                let self_field = if is_ty_option(&field.ty) {
                    let db_field = table.columns.iter().find(|column| column.name == field.name).unwrap();
                    match &db_field.default {
                        Some(DatabaseDefault::Bool(b)) => quote!(self.#field_ident.unwrap_or(#b)),
                        Some(DatabaseDefault::Float(f)) => {
                            let f = Literal::i64_unsuffixed(*f);
                            quote!(self.#field_ident.unwrap_or(#f))
                        },
                        Some(DatabaseDefault::Int(i)) => {
                            let i = Literal::u64_unsuffixed(*i);
                            quote!(self.#field_ident.unwrap_or(#i))
                        },
                        Some(DatabaseDefault::String(s)) => quote!(self.#field_ident.unwrap_or(#s)),
                        _ => quote!(self.#field_ident),
                    }
                } else {
                    quote!(self.#field_ident)
                };

                quote!(
                    #field_ident: ::sea_orm::entity::IntoActiveValue::into_active_value(#self_field).into()
                )
            });
            
            let expanded = quote!(
                impl ::sea_orm::entity::IntoActiveModel<crate::#db_module_ident::ActiveModel> for ::schema::#ident {
                    fn into_active_model(self) -> crate::#db_module_ident::ActiveModel {
                        crate::#db_module_ident::ActiveModel {
                            #( #active_values, )*
                            ..Default::default()
                        }
                    }
                }
            );

            write!(code, "{}", expanded.to_string()).unwrap();
        }

        code.trim().to_string()
    }

    fn database_tables(&self) -> Vec<(&Model, &DatabaseTable)> {
        self.models.iter().fold(Vec::new(), |mut acc, model| {
            let roles = model
                .roles
                .iter()
                .filter_map(|role| match role {
                    Role::DatabaseTable(database_table) => Some((model, database_table)),
                    _ => None,
                })
                .collect::<Vec<_>>();

            acc.extend(roles);

            acc
        })
    }

    fn database_sub_tables(&self) -> Vec<(&Model, &DatabaseTable)> {
        self.models.iter().fold(Vec::new(), |mut acc, model| {
            let roles = model
                .roles
                .iter()
                .filter_map(|role| match role {
                    Role::DatabaseSubTable(database_sub_table) => Some((model, database_sub_table)),
                    _ => None,
                })
                .collect::<Vec<_>>();

            acc.extend(roles);

            acc
        })
    }

    async fn fetch_table(
        &self,
        table: &DatabaseTable,
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
            reference: Option<String>,
        }

        let raw_columns: Vec<ColumnsQuery> = sqlx::query_as(FETCH_TABLE_QUERY)
            .bind("public")
            .bind(&table.name)
            .fetch_all(&*self.pool)
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
                        .map_err(|_| Error::UnsupportedType(table.name.clone(), column_name))?,
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

    fn write_table_create_sql(&self, table: &DatabaseTable) -> String {
        let mut sql = String::new();

        writeln!(sql, "CREATE TABLE IF NOT EXISTS {} (", table.name).unwrap();

        for (i, column) in table.columns.iter().enumerate() {
            write!(sql, "  {}", self.write_column_sql(column)).unwrap();

            if i < table.columns.len() - 1 {
                writeln!(sql, ",").unwrap();
            } else {
                writeln!(sql).unwrap();
            }
        }

        writeln!(sql, ");").unwrap();

        sql
    }

    fn write_column_sql(&self, column: &DatabaseColumn) -> String {
        let mut sql = String::new();

        write!(sql, "{} {}", column.name, column.ty,).unwrap();

        if !column.nullable {
            write!(sql, " NOT NULL",).unwrap();
        }

        if let Some(default) = &column.default {
            write!(sql, " DEFAULT {}", default).unwrap();
        }

        if let Some(constraint) = &column.constraint {
            write!(sql, " CHECK ({})", constraint).unwrap();
        }

        if column.primary_key {
            write!(sql, " PRIMARY KEY").unwrap();
        }

        if let Some((table, col)) = &column.references {
            write!(sql, " REFERENCES {}({})", table, col).unwrap();
        }

        sql
    }

    async fn write_sync_sql(&self, table: &DatabaseTable, db_columns: &[DatabaseColumn]) -> String {
        let mut sql = String::new();

        for schema_col in &table.columns {
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
                        table.name,
                        self.write_column_sql(schema_col)
                    )
                    .unwrap();
                    continue;
                }
            };

            // Check for type mismatch
            if schema_col.ty != db_col.ty {
                writeln!(
                    sql,
                    "ALTER TABLE {table} ALTER COLUMN {column} TYPE {ty} USING {column}::{ty};",
                    table = table.name,
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
                        table = table.name,
                        column = schema_col.name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        sql,
                        "ALTER TABLE {table} ALTER COLUMN {column} DROP NOT NULL;",
                        table = table.name,
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
                        table = table.name,
                        column = schema_col.name,
                        default = default
                    )
                    .unwrap();
                } else {
                    writeln!(
                        sql,
                        "ALTER TABLE {table} ALTER COLUMN {column} DROP DEFAULT;",
                        table = table.name,
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
                        table = table.name,
                        column = schema_col.name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        sql,
                        "ALTER TABLE {table} ADD CONSTRAINT {table}_{column}_key UNIQUE ({column});",
                        table = table.name,
                        column = schema_col.name
                    )
                    .unwrap();
                }
            }

            // Check for references mismatch
            if schema_col.references != db_col.references {
                if let Some(references) = &schema_col.references {
                    if db_col.references.is_some() {
                        writeln!(
                            sql,
                            "ALTER TABLE {table} DROP CONSTRAINT {table}_{column}_fkey;",
                            table = table.name,
                            column = schema_col.name
                        )
                        .unwrap();
                    }
                    writeln!(
                        sql,
                        "ALTER TABLE {table} ADD CONSTRAINT {table}_{column}_fkey FOREIGN KEY ({column}) REFERENCES {reference_table} ({reference_column});",
                        table = table.name,
                        column = schema_col.name,
                        reference_table = references.0,
                        reference_column = references.1,
                    )
                    .unwrap();
                } else {
                    writeln!(
                        sql,
                        "ALTER TABLE {table} DROP CONSTRAINT {table}_{column}_fkey;",
                        table = table.name,
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
                table
                    .columns
                    .iter()
                    .all(|schema_col| schema_col.name != db_col.name)
            })
            .for_each(|db_col| {
                writeln!(
                    sql,
                    "ALTER TABLE {table} DROP COLUMN {column};",
                    table = table.name,
                    column = db_col.name
                )
                .unwrap();
            });

        sql
    }
}

const FETCH_TABLE_QUERY: &str = "
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
";
