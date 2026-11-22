mod tool;

pub use tool::{FFmpeg, FFprobe};

#[derive(Debug)]
pub struct VideoProcessor {
    pub ffmpeg: FFmpeg,
    pub ffprobe: FFprobe,
}
