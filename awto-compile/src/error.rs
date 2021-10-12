#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database has unsupported type in {0}.{0}")]
    UnsupportedType(String, String),
    #[error("database error: {0}")]
    Sqlx(sqlx::Error),
}
