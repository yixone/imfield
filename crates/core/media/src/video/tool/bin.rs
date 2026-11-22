use std::path::PathBuf;

use tokio::process::Command;

/// Specifies the executable binary used by the application
#[derive(Debug)]
pub(crate) enum ExecutableBinary {
    /// Uses the binary resolved through the system `PATH`
    SystemPath(String),

    // TODO: add `f_lock` for bin file
    /// Uses the binary at the specified filesystem path
    File(PathBuf),
}

impl ExecutableBinary {
    /// Creates a new command configured to execute the binary
    pub fn command(&self) -> Command {
        match self {
            ExecutableBinary::SystemPath(path) => Command::new(path),
            ExecutableBinary::File(path) => Command::new(path),
        }
    }
}
