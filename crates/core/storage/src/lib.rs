//! Storage implementation for Imfield

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!("Unsupported operating system");

pub mod files;
pub mod system;

pub mod error;
pub mod fs;
pub mod layout;
pub mod storage;

pub use error::StorageError;
