use std::time::Duration;

#[derive(Debug)]
pub struct VideoStreamData {
    pub codec: VideoCodec,
    pub dimension: (u32, u32),
    pub fps: f64,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct AudioStreamData {
    pub codec: AudioCodec,
}

#[derive(Debug)]
pub struct VideoMetadata {
    pub video: VideoStreamData,
    pub audio: Option<AudioStreamData>,
}

#[derive(Debug)]
pub enum VideoCodec {
    H264,
    Av1,
    Other(String),
}

impl From<String> for VideoCodec {
    fn from(s: String) -> Self {
        match s.as_str() {
            "h264" => VideoCodec::H264,
            "av1" => VideoCodec::Av1,
            _ => VideoCodec::Other(s),
        }
    }
}

#[derive(Debug)]
pub enum AudioCodec {
    Aac,
    Mp3,
    Flac,
    Other(String),
}

impl From<String> for AudioCodec {
    fn from(s: String) -> Self {
        match s.as_str() {
            "aac" => AudioCodec::Aac,
            "mp3" => AudioCodec::Mp3,
            "flac" => AudioCodec::Flac,
            _ => AudioCodec::Other(s),
        }
    }
}

pub enum AudioMode {
    Disabled,
    Auto,
}
