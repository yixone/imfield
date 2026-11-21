use std::time::Duration;

use imfield_jobs_core::ExecutableJob;
use imfield_models::types::id::AssetId;
use tokio_util::sync::CancellationToken;

pub enum JobV1 {
    ProcessAsset { id: AssetId },
}

#[async_trait::async_trait]
impl ExecutableJob for JobV1 {
    type Context = ();

    type Error = ();

    fn name(&self) -> &'static str {
        match self {
            JobV1::ProcessAsset { .. } => "job.asset.process",
        }
    }

    fn max_retries(&self) -> u32 {
        16
    }

    fn retry_delay(&self) -> Option<Duration> {
        Some(Duration::from_mins(5))
    }

    async fn execute(
        &self,
        ctx: &Self::Context,
        cancel: CancellationToken,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
