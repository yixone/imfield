use std::time::Duration;

pub struct VideoStreamData {
    pub codec: VideoCodec,
    pub dimension: (u32, u32),
    pub duration: Duration,
}

pub struct AudioStreamData {
    pub codec: AudioCodec,
}

pub struct VideoMetadata {
    pub video: VideoStreamData,
    pub audio: Option<AudioStreamData>,
}

pub enum VideoCodec {
    H264,
    Av1,
    Other(String),
}

pub enum AudioCodec {
    Aac,
    Mp3,
    Flac,
    Other(String),
}

pub enum AudioMode {
    Disabled,
    Auto,
}
