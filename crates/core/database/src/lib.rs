//! Database access layer
//!
//! Defines database access abstractions and provides an SQLite
//! implementation for application data persistence

pub mod ports;
pub mod types;

pub mod error;

pub mod driver;
pub mod impls;

mod schemas;

pub use driver::SqliteDatabase;
pub use error::DatabaseError;
