use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;

use crate::video::tool::{
    VideoToolError,
    bin::{CommandExt, ExecutableBinary},
    types::{AudioStreamData, VideoMetadata, VideoStreamData},
};

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

        let out = command.execute_to_str().await.map_err(|e| match e {
            VideoToolError::Io(..) => VideoToolError::BinUnavailable,
            _ => e,
        })?;

        let mut seg = out.split_whitespace();
        let version = match (seg.next(), seg.next(), seg.next()) {
            (Some("ffprobe"), Some("version"), Some(version)) => version.to_string(),
            _ => return Err(VideoToolError::BinUnavailable),
        };

        Ok(FFprobe { bin, version })
    }

    pub async fn probe_meta(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<VideoMetadata, VideoToolError> {
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
        command.arg(path.as_ref());

        // Execute the command and read the output in JSON format
        let out = command.execute_to_buf().await?;
        let json = serde_json::from_slice::<ProbedDataJson>(&out)
            .map_err(|_| VideoToolError::OutputDecodeFailed)?;

        let streams = json.streams;

        // Receives a video stream
        let vs_json = streams
            .iter()
            .find(|d| d.is_video())
            .ok_or(VideoToolError::MissingVideoStream)?;
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
    pub fn is_video(&self) -> bool {
        &self.codec_type == "video"
    }

    pub fn is_audio(&self) -> bool {
        &self.codec_type == "audio"
    }

    pub fn dimension(&self) -> Result<(u32, u32), VideoToolError> {
        Ok((
            self.width.ok_or(VideoToolError::OutputDecodeFailed)?,
            self.height.ok_or(VideoToolError::OutputDecodeFailed)?,
        ))
    }

    pub fn fps(&self) -> Result<f64, VideoToolError> {
        let Some(frame_rate) = &self.avg_frame_rate else {
            return Err(VideoToolError::OutputDecodeFailed);
        };

        let Some((num, denom)) = frame_rate.split_once('/') else {
            return Err(VideoToolError::OutputDecodeFailed);
        };

        let num = num
            .parse::<f64>()
            .map_err(|_| VideoToolError::OutputDecodeFailed)?;
        let den = denom
            .parse::<f64>()
            .map_err(|_| VideoToolError::OutputDecodeFailed)?;

        if num.is_infinite() || den.is_infinite() || den <= 0.0 {
            return Err(VideoToolError::OutputDecodeFailed);
        }

        Ok(num / den)
    }

    pub fn duration(&self) -> Result<Duration, VideoToolError> {
        let duration_secs = self
            .duration
            .parse::<f32>()
            .map_err(|_| VideoToolError::OutputDecodeFailed)?;

        if duration_secs.is_infinite() || duration_secs < 0.0 {
            return Err(VideoToolError::OutputDecodeFailed);
        }

        Ok(Duration::from_secs_f32(duration_secs))
    }
}
