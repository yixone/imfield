//! SQLite implementation of the database access layer
//!
//! Provides database operations, schema management, and persistence
//! logic using SQLite

pub mod impls;
pub mod schemas;

pub mod driver;

pub use driver::SqliteDatabase;
