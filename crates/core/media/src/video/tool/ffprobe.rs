use std::path::PathBuf;

use crate::video::tool::{VideoToolError, bin::ExecutableBinary};

const FFPROBE_SYSTEM_PATH: &str = "ffprobe";

#[derive(Debug)]
pub struct FFprobe {
    bin: ExecutableBinary,
    version: String,
}

impl FFprobe {
    pub async fn try_new() -> Result<Self, VideoToolError> {
        FFprobe::from_bin(ExecutableBinary::SystemPath(FFPROBE_SYSTEM_PATH.into())).await
    }

    pub async fn try_new_file(path: impl Into<PathBuf>) -> Result<Self, VideoToolError> {
        FFprobe::from_bin(ExecutableBinary::File(path.into())).await
    }

    async fn from_bin(bin: ExecutableBinary) -> Result<Self, VideoToolError> {
        let mut command = bin.command();
        command.arg("-version");

        // Check that the command is being executed
        let Ok(out) = command.output().await else {
            return Err(VideoToolError::BinUnavailable);
        };
        if !out.status.success() {
            return Err(VideoToolError::BinUnavailable);
        }

        // Get the ffmpeg version
        let out = String::from_utf8(out.stdout).map_err(|_| VideoToolError::StdoutDecodeFailed)?;
        let mut seg = out.split_whitespace();
        let version = match (seg.next(), seg.next(), seg.next()) {
            (Some("ffmpeg"), Some("version"), Some(version)) => version.to_string(),
            _ => return Err(VideoToolError::BinUnavailable),
        };

        Ok(FFprobe { bin, version })
    }

    pub async fn probe_meta(&self, path: impl Into<PathBuf>) {
        todo!()
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
