use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::video::{
    VideoError,
    execution::{CommandExt, ExecutableBinary},
};

const FFMPEG_SYSTEM_PATH: &str = "ffmpeg";

/// Wrapper around the FFmpeg executable
///
/// `FFmpeg` provides methods for extracting frames from video files and
/// creating video fragments
///
/// The executable can be resolved through the system `PATH` or
/// specified explicitly
#[derive(Debug)]
pub struct FFmpeg {
    bin: ExecutableBinary,
    version: String,
}

impl FFmpeg {
    /// Creates an FFmpeg instance using the executable resolved through the
    /// system `PATH`
    pub async fn try_new() -> Result<Self, VideoError> {
        FFmpeg::from_bin(ExecutableBinary::SystemPath(FFMPEG_SYSTEM_PATH.into())).await
    }

    /// Creates an FFmpeg instance using an executable at the specified path
    pub async fn try_new_file(path: impl Into<PathBuf>) -> Result<Self, VideoError> {
        FFmpeg::from_bin(ExecutableBinary::File(path.into())).await
    }

    /// Extracts a frame from a video at the specified timestamp
    ///
    /// The resulting frame is encoded as JPEG and returned as a buffer
    pub async fn read_frame(
        &self,
        path: impl AsRef<Path>,
        frame_time: Duration,
    ) -> Result<Vec<u8>, VideoError> {
        let mut command = self.bin.command();

        command.arg("-i");
        command.arg(path.as_ref());

        command.args(["-ss", &frame_time.as_secs_f32().to_string()]);

        command.args(["-f", "image2pipe"]);
        command.args(["-vcodec", "mjpeg"]);
        command.arg("pipe:1");

        let buf = command.execute_to_buf().await?;

        Ok(buf)
    }

    /// Extracts a video fragment and saves it to the specified output path
    ///
    /// The fragment is extracted from the source video starting at
    /// [`ExtractVideoFragmentParams::start_at`] and limited to the specified
    /// duration
    ///
    /// The output is encoded using H.264 and stored in an MP4 container
    pub async fn extract_video_fragment(
        &self,
        path: impl AsRef<Path>,
        output: impl AsRef<Path>,
        params: ExtractVideoFragmentParams,
    ) -> Result<(), VideoError> {
        let mut command = self.bin.command();

        let output = output.as_ref();
        if tokio::fs::try_exists(output).await? {
            return Err(VideoError::FileAlreadyExists);
        }

        command.arg("-i");
        command.arg(path.as_ref());

        command.args(["-ss", &params.start_at.as_secs_f32().to_string()]);
        command.args(["-t", &params.duration.as_secs_f32().to_string()]);

        if let Some(fps) = params.framerate {
            command.args(["-r", &fps.to_string()]);
        }

        match params.audio {
            AudioMode::Auto => (),
            AudioMode::Disable => {
                command.arg("-an");
            }
        }

        match params.resize {
            ResizeMode::Auto => {}
            ResizeMode::ByWidth(w) => {
                command.args(["-vf", &format!("scale={w}:-2")]);
            }
        }

        command.args(["-c:v", "libx264"]);

        command.args(["-f", "mp4"]);
        command.arg(output);

        command.execute().await?;
        Ok(())
    }

    /// Returns the version of the FFmpeg executable
    pub fn version(&self) -> &str {
        &self.version
    }

    async fn from_bin(bin: ExecutableBinary) -> Result<Self, VideoError> {
        let mut command = bin.command();
        command.arg("-version");

        let out = command.execute_to_str().await.map_err(|e| match e {
            VideoError::Io(..) => VideoError::BinUnavailable,
            _ => e,
        })?;

        let mut seg = out.split_whitespace();
        let version = match (seg.next(), seg.next(), seg.next()) {
            (Some("ffmpeg"), Some("version"), Some(version)) => version.to_string(),
            _ => return Err(VideoError::BinUnavailable),
        };

        Ok(FFmpeg { bin, version })
    }
}

/// Controls the dimensions of the extracted video fragment
pub enum ResizeMode {
    /// Preserves the source video dimension
    Auto,

    /// Resizes the video to the specified width while preserving
    /// its aspect ratio
    ByWidth(u32),
}

/// Controls how the audio stream is handling during video fragment extraction
pub enum AudioMode {
    /// Automatically selects the audio handling behaviour of FFmpeg
    Auto,

    /// Disables audio in the output fragment
    Disable,
}

/// Parameters used when extracing and encoding a video fragment
pub struct ExtractVideoFragmentParams {
    /// Timestamp at which the extraction starts
    pub start_at: Duration,

    /// Maximum duration of the extracted fragments
    pub duration: Duration,

    /// Output frame rate
    pub framerate: Option<u32>,

    /// Controls whether audio is included in the output
    pub audio: AudioMode,

    /// Controls the output video dimension
    pub resize: ResizeMode,
}
