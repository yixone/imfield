//! Database access abstractions and contracts
//!
//! Defines the interfaces and common types used by the application
//! to interact with persistent data storage

pub mod ports;
pub mod types;

pub mod error;

pub use error::DatabaseError;
