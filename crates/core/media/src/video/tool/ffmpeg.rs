use std::path::PathBuf;

use crate::video::tool::{
    bin::{CommandExt, ExecutableBinary},
    error::VideoToolError,
};

const FFMPEG_SYSTEM_PATH: &str = "ffmpeg";

#[derive(Debug)]
pub struct FFmpeg {
    bin: ExecutableBinary,
    version: String,
}

impl FFmpeg {
    pub async fn try_new() -> Result<Self, VideoToolError> {
        FFmpeg::from_bin(ExecutableBinary::SystemPath(FFMPEG_SYSTEM_PATH.into())).await
    }

    pub async fn try_new_file(path: impl Into<PathBuf>) -> Result<Self, VideoToolError> {
        FFmpeg::from_bin(ExecutableBinary::File(path.into())).await
    }

    async fn from_bin(bin: ExecutableBinary) -> Result<Self, VideoToolError> {
        let mut command = bin.command();
        command.arg("-version");

        let out = command.execute_to_str().await.map_err(|e| match e {
            VideoToolError::Io(..) => VideoToolError::BinUnavailable,
            _ => e,
        })?;

        let mut seg = out.split_whitespace();
        let version = match (seg.next(), seg.next(), seg.next()) {
            (Some("ffmpeg"), Some("version"), Some(version)) => version.to_string(),
            _ => return Err(VideoToolError::BinUnavailable),
        };

        Ok(FFmpeg { bin, version })
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
