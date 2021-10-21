//! <div align="center">
//!   <h1>awto</h1>
//!
//!   <p>
//!     <strong>Awtomate your 🦀 microservices with awto</strong>
//!   </p>
//!
//! </div>
//!
//! # Awto
//!
//! Awto treats your rust project as the source of truth for microservices,
//! and generates **database tables** and **protobufs** based on your schema and service.
//!
//! See more on the [repository](https://github.com/awto-rs/awto).

pub use awto_macros as macros;
pub use lazy_static;

pub mod database;
pub mod prelude;
pub mod protobuf;
pub mod schema;
pub mod service;
#[doc(hidden)]
pub mod tests_cfg;
