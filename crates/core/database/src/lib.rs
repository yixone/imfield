//! Database access layer
//!
//! Defines database access abstractions and provides an SQLite
//! implementation for application data persistence

pub mod impls;
pub mod ports;
pub mod types;

pub(crate) mod schemas;
pub(crate) mod utils;

pub mod driver;
pub mod error;

pub use driver::SqliteDatabase;
pub use error::DatabaseError;
