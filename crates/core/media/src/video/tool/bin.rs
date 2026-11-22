use std::{path::PathBuf, process::Output};

use tokio::process::Command;

use crate::video::tool::VideoToolError;

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

#[allow(async_fn_in_trait)]
pub(in crate::video) trait CommandExt {
    async fn execute(&mut self) -> Result<Output, VideoToolError>;

    async fn execute_to_str(&mut self) -> Result<String, VideoToolError>;

    async fn execute_to_buf(&mut self) -> Result<Vec<u8>, VideoToolError>;
}

impl CommandExt for Command {
    async fn execute(&mut self) -> Result<Output, VideoToolError> {
        let out = self.output().await.map_err(VideoToolError::Io)?;

        if !out.status.success() {
            let err =
                String::from_utf8(out.stderr).map_err(|_| VideoToolError::OutputDecodeFailed)?;
            return Err(VideoToolError::Io(std::io::Error::other(err)));
        }

        Ok(out)
    }

    async fn execute_to_buf(&mut self) -> Result<Vec<u8>, VideoToolError> {
        let out = self.execute().await?;
        Ok(out.stdout)
    }

    async fn execute_to_str(&mut self) -> Result<String, VideoToolError> {
        let out = self.execute_to_buf().await?;
        String::from_utf8(out).map_err(|_| VideoToolError::OutputDecodeFailed)
    }
}
