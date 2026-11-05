//! Database access layer
//!
//! Defines database access abstractions and provides an SQLite
//! implementation for application data persistence

pub mod ports;
pub mod types;

pub mod impls;

pub mod driver;
pub mod error;

mod schemas;
mod utils;

pub use driver::SqliteDatabase;
pub use error::DatabaseError;
