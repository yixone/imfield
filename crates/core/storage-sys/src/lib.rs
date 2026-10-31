//! Low-level utilities for file storage

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!("Unsupported operating system");

pub mod rename;
pub mod stats;

pub use rename::rename_exclusive;
pub use stats::statvfs;
