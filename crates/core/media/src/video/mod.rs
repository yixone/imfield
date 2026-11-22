mod error;

mod execution;
mod ffmpeg;
mod ffprobe;
mod types;

pub use error::VideoError;
pub use ffmpeg::{AudioMode, ExtractVideoFragmentParams, FFmpeg, ResizeMode};
pub use ffprobe::FFprobe;

#[derive(Debug)]
pub struct VideoProcessor {
    pub ffmpeg: FFmpeg,
    pub ffprobe: FFprobe,
}
