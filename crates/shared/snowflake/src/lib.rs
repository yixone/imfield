//! Implementation of the snowflake-id algorithm for Imfield
//!
//! ### Usage
//! ```
//! use imfield_snowflake::SnowflakeGenerator;
//!
//! // Define a generator for Snowflake ID generation
//! let generator = SnowflakeGenerator::new(0);
//!
//! // Generates an ID using the generator
//! let id = generator.get_id();
//!
//! // Generates a string ID using the generator
//! let id_str = generator.get_id_str();
//! ```
//!
//! ### Crate features
//! - `sqlx`: Enables [`sqlx::Type`] implementations for types
//! - `serde`: Enables [`serde::Serialize`] and [`serde::Deserialize`] implementations for types

mod generator;
mod id;

pub use generator::SnowflakeGenerator;
pub use id::{SnowflakeId, SnowflakeIdStr};
