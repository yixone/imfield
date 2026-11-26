use std::{sync::Arc, time::Duration};

use imfield_di::DataContext;
use imfield_jobs_core::ExecutableJob;
use imfield_media::video::{FFmpeg, FFprobe};
use imfield_models::types::id::AssetId;
use tokio_util::sync::CancellationToken;

mod commands;
mod utils;

mod error;

const MAX_RETRIES: u32 = 16;

#[derive(Debug)]
pub enum JobV1 {
    ProcessAsset { id: AssetId },

    ProcessPendingAssetsBatch,
}

#[async_trait::async_trait]
impl ExecutableJob for JobV1 {
    type Context = JobContext;

    type Error = error::JobError;

    fn name(&self) -> &'static str {
        match self {
            JobV1::ProcessAsset { .. } => "job.asset.process",
            JobV1::ProcessPendingAssetsBatch => "job.asset.process.batch",
        }
    }

    fn max_retries(&self) -> u32 {
        MAX_RETRIES
    }

    fn retry_delay(&self) -> Option<Duration> {
        Some(Duration::from_mins(5))
    }

    async fn execute(
        &self,
        ctx: &Self::Context,
        cancel: CancellationToken,
    ) -> Result<(), Self::Error> {
        match self {
            JobV1::ProcessAsset { id } => commands::process_asset(*id, ctx, cancel).await,
            JobV1::ProcessPendingAssetsBatch => {
                commands::process_pending_assets_batch(ctx, cancel).await
            }
        }
    }
}

pub struct VideoContext {
    pub ffmpeg: FFmpeg,
    pub ffprobe: FFprobe,
}

pub struct JobContext {
    pub video: Option<Arc<VideoContext>>,
    pub data: Arc<DataContext>,
}
