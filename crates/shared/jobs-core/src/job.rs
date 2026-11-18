use std::{fmt::Debug, time::Duration};

use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::utils;

#[async_trait::async_trait]
pub trait ExecutableJob: Send + Sync {
    type Context: Clone + Send + Sync + 'static;
    type Error: Debug + Send + Sync;

    fn name(&self) -> &'static str;

    fn allow_concurrency(&self) -> bool {
        true
    }

    fn max_retries(&self) -> u32;
    fn retry_delay(&self) -> Option<Duration> {
        None
    }

    async fn execute(&self, ctx: &Self::Context) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(imfield_snowflake::SnowflakeId);

impl From<imfield_snowflake::SnowflakeId> for JobId {
    fn from(id: imfield_snowflake::SnowflakeId) -> Self {
        JobId(id)
    }
}

#[derive(Clone)]
pub struct JobItem<E: ExecutableJob> {
    id: JobId,
    data: E,
    job_type: JobType,
    retries: u32,
    retry_delay: Option<Duration>,
    created_at: i64,
}

impl<E> JobItem<E>
where
    E: ExecutableJob,
{
    pub fn new(id: JobId, job_type: JobType, data: E) -> Self {
        JobItem {
            id,
            job_type,
            retry_delay: data.retry_delay(),
            created_at: utils::get_now_ms(),
            retries: 0,
            data,
        }
    }

    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.data.name()
    }

    pub fn allow_concurrency(&self) -> bool {
        self.data.allow_concurrency()
    }

    pub fn max_retries(&self) -> u32 {
        self.data.max_retries()
    }

    pub async fn execute(&self, ctx: &E::Context) -> Result<(), E::Error> {
        self.data.execute(ctx).await
    }

    pub fn retries(&self) -> u32 {
        self.retries
    }

    pub fn add_retry(&mut self) -> u32 {
        self.retries += 1;
        self.retries
    }

    pub fn reset_retries(&mut self) {
        self.retries = 0;
    }

    pub fn retry_delay(&self) -> Option<Duration> {
        self.retry_delay
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    pub fn job_type(&self) -> JobType {
        self.job_type
    }

    pub fn set_job_type(&mut self, job_type: JobType) {
        self.job_type = job_type;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JobType {
    Normal,
    Schedule {
        scheduled_at: Instant,
    },
    Interval {
        next_run_at: Instant,
        interval: Duration,
    },
}

pub enum InactiveJobStatus<E>
where
    E: ExecutableJob,
{
    Finished,
    Failed(E::Error),
    Unknown,
}

#[derive(Clone)]
pub struct ActiveJob<E>
where
    E: ExecutableJob,
{
    cancel: CancellationToken,
    runned_at: Instant,
    pub job: JobItem<E>,
}

impl<E> ActiveJob<E>
where
    E: ExecutableJob,
{
    pub fn new(job: JobItem<E>) -> Self {
        ActiveJob {
            cancel: CancellationToken::new(),
            runned_at: Instant::now(),
            job,
        }
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }
}
