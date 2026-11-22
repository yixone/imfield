use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;

use crate::video::{
    VideoError,
    tool::{
        bin::{CommandExt, ExecutableBinary},
        types::{AudioStreamData, VideoMetadata, VideoStreamData},
    },
};

const FFPROBE_SYSTEM_PATH: &str = "ffprobe";

/// Wrapper around the FFprobe executable
///
/// `FFprobe` provide access to media metadata through the FFprobe CLI.
/// The executable can be resolved through the system `PATH` or
/// specified explicitly
#[derive(Debug)]
pub struct FFprobe {
    bin: ExecutableBinary,
    version: String,
}

impl FFprobe {
    /// Creates an FFprobe instance using the executable resolved through the
    /// system `PATH`
    pub async fn try_new() -> Result<Self, VideoError> {
        FFprobe::from_bin(ExecutableBinary::SystemPath(FFPROBE_SYSTEM_PATH.into())).await
    }

    /// Creates an FFprobe instance using an executable at the specified path
    pub async fn try_new_file(path: impl Into<PathBuf>) -> Result<Self, VideoError> {
        FFprobe::from_bin(ExecutableBinary::File(path.into())).await
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
            (Some("ffprobe"), Some("version"), Some(version)) => version.to_string(),
            _ => return Err(VideoError::BinUnavailable),
        };

        Ok(FFprobe { bin, version })
    }

    /// Extract metadata from a video file
    ///
    /// Retrieves information about the first video stream and, if present, the first audio stream
    pub async fn probe_meta(&self, path: impl AsRef<Path>) -> Result<VideoMetadata, VideoError> {
        // Check if the file exists
        let path = path.as_ref();
        if !tokio::fs::try_exists(&path).await.map_err(VideoError::Io)? {
            return Err(VideoError::FileNotFound);
        }

        // Construct a command to read metadata
        let mut command = self.bin.command();
        command.arg("-hide_banner");
        command.args(["-v", "error"]);
        command.arg("-show_streams");
        command.args([
            "-show_entries",
            "stream=codec_name,codec_type,width,height,avg_frame_rate,duration",
        ]);
        command.args(["-of", "json"]);

        command.arg("-i");
        command.arg(path);

        // Execute the command and read the output in JSON format
        let out = command.execute_to_buf().await?;
        let json =
            serde_json::from_slice::<ProbedDataJson>(&out).map_err(VideoError::DeserializeError)?;

        let streams = json.streams;

        // Receives a video stream
        let vs_json = streams
            .iter()
            .find(|d| d.is_video())
            .ok_or(VideoError::MissingStream)?;
        let video = VideoStreamData {
            codec: vs_json.codec_name.to_string().into(),
            dimension: vs_json.dimension()?,
            fps: vs_json.fps()?,
            duration: vs_json.duration()?,
        };

        // Receives an audio stream
        let as_json = streams.iter().find(|d| d.is_audio());
        let audio = as_json.map(|j| AudioStreamData {
            codec: j.codec_name.to_string().into(),
        });

        Ok(VideoMetadata { video, audio })
    }

    /// Returns the version of the FFprobe executable
    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, Deserialize)]
struct ProbedDataJson {
    streams: Vec<StreamDataJson>,
}

#[derive(Debug, Deserialize)]
struct StreamDataJson {
    codec_name: String,
    codec_type: String,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    duration: String,
}

impl StreamDataJson {
    fn is_video(&self) -> bool {
        &self.codec_type == "video"
    }

    fn is_audio(&self) -> bool {
        &self.codec_type == "audio"
    }

    fn dimension(&self) -> Result<(u32, u32), VideoError> {
        Ok((
            self.width.ok_or(VideoError::MissingMetadata)?,
            self.height.ok_or(VideoError::MissingMetadata)?,
        ))
    }

    fn fps(&self) -> Result<f64, VideoError> {
        let Some(frame_rate) = &self.avg_frame_rate else {
            return Err(VideoError::MissingMetadata);
        };

        let Some((num, denom)) = frame_rate.split_once('/') else {
            return Err(VideoError::DecodeFailed);
        };

        let num = num.parse::<f64>().map_err(|_| VideoError::DecodeFailed)?;
        let den = denom.parse::<f64>().map_err(|_| VideoError::DecodeFailed)?;

        if !num.is_finite() || !den.is_finite() || den <= 0.0 {
            return Err(VideoError::DecodeFailed);
        }

        Ok(num / den)
    }

    fn duration(&self) -> Result<Duration, VideoError> {
        let duration_secs = self
            .duration
            .parse::<f32>()
            .map_err(|_| VideoError::DecodeFailed)?;

        if !duration_secs.is_finite() || duration_secs < 0.0 {
            return Err(VideoError::DecodeFailed);
        }

        Ok(Duration::from_secs_f32(duration_secs))
    }
}
