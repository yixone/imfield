mod error;

mod tool;

pub use error::VideoError;
pub use tool::{FFmpeg, FFprobe};

#[derive(Debug)]
pub struct VideoProcessor {
    pub ffmpeg: FFmpeg,
    pub ffprobe: FFprobe,
}
