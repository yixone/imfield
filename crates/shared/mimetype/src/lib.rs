//! Types and utilites for media type identification
//!
//! Module provides MIME types representations and detectors for
//! identifying media types from file magic bytes

mod detector;
mod types;

pub use detector::guess_mime;
pub use types::{MimeKind, MimeType};
