use std::fmt::Debug;

use chrono::{DateTime, Duration, Utc};
use tokio_util::sync::CancellationToken;

use crate::utils;

#[async_trait::async_trait]
pub trait ExecutableJob {
    type Context: Clone + Send + Sync + 'static;
    type Error: Debug;

    fn name(&self) -> &'static str;

    fn allow_concurrency(&self) -> bool {
        true
    }

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

pub struct JobItem<E: ExecutableJob> {
    id: JobId,
    data: E,
    retries: u32,
    retry_delay: Option<Duration>,
    created_at: i64,
}

impl<E> JobItem<E>
where
    E: ExecutableJob,
{
    pub fn new(id: JobId, data: E) -> Self {
        JobItem {
            id,
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

    #[inline]
    pub fn can_retry(&self) -> bool {
        self.retry_delay().is_some()
    }

    pub fn retry_delay(&self) -> Option<Duration> {
        self.retry_delay
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JobType {
    Normal,
    Schedule {
        scheduled_at: DateTime<Utc>,
    },
    Interval {
        next_run_at: DateTime<Utc>,
        interval: Duration,
    },
}

pub enum InactiveJobStatus {
    Finished,
    Failed,
    Unknown,
}

pub struct ActiveJob<E>
where
    E: ExecutableJob,
{
    cancel: CancellationToken,
    runned_at: i64,
    job: JobItem<E>,
}

impl<E> ActiveJob<E>
where
    E: ExecutableJob,
{
    pub fn item(&self) -> &JobItem<E> {
        &self.job
    }
}
