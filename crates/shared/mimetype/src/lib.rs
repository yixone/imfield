//! MIME type detection crate
//!
//! Crate provides utilites for
//! identifying media types from file magic bytes

mod detector;
mod types;

pub use detector::guess_mime;
pub use types::{MimeKind, MimeType};
