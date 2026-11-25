use std::time::Duration;

#[derive(Debug)]
pub struct VideoMetadata {
    pub(crate) video: VideoStreamData,
    pub(crate) audio: Option<AudioStreamData>,
}

impl VideoMetadata {
    pub fn video(&self) -> &VideoStreamData {
        &self.video
    }

    pub fn audio(&self) -> &Option<AudioStreamData> {
        &self.audio
    }

    pub fn has_audio(&self) -> bool {
        self.audio().is_some()
    }
}

#[derive(Debug)]
pub struct VideoStreamData {
    pub(crate) codec: VideoCodec,
    pub(crate) dimension: (u32, u32),
    pub(crate) fps: f64,
    pub(crate) duration: Duration,
}

impl VideoStreamData {
    pub fn codec(&self) -> &VideoCodec {
        &self.codec
    }

    pub fn width(&self) -> u32 {
        self.dimension.0
    }

    pub fn height(&self) -> u32 {
        self.dimension.1
    }

    pub fn fps(&self) -> f64 {
        self.fps
    }

    #[inline]
    pub fn fps_int(&self) -> u32 {
        self.fps.round() as u32
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    #[inline]
    pub fn duration_ms(&self) -> u64 {
        self.duration().as_millis() as u64
    }
}

#[derive(Debug)]
pub struct AudioStreamData {
    pub(crate) codec: AudioCodec,
}

impl AudioStreamData {
    pub fn codec(&self) -> &AudioCodec {
        &self.codec
    }
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
