//! Application models and value types
//!
//! Defines the data structures and types shared across application
//! components

pub mod entities;
pub mod types;
pub mod utils;

pub mod error;
pub mod events;

pub use error::DomainError;
