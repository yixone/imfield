use std::fmt::Debug;

use chrono::{DateTime, Duration, Utc};

use crate::utils;

#[async_trait::async_trait]
pub trait ExecutableJob {
    type Context: Clone + Send + Sync + 'static;
    type Error: Debug;

    fn name(&self) -> &'static str;
    fn retry_strategy(&self) -> Option<RetryStrategy> {
        None
    }

    async fn execute(&self, ctx: &Self::Context) -> Result<(), Self::Error>;
}

pub struct Job<E: ExecutableJob> {
    pub id: JobId,

    pub retry: Option<JobRetry>,
    pub state: JobState,
    pub job_type: JobType,

    pub data: E,

    pub created_at: i64,
    pub finished_at: Option<i64>,
}

impl<E> Job<E>
where
    E: ExecutableJob,
{
    pub fn new(id: JobId, job_type: JobType, retry: Option<JobRetry>, data: E) -> Self {
        Job {
            id,
            retry,
            state: JobState::Queued,
            job_type,
            data,
            created_at: utils::get_now_ms(),
            finished_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JobState {
    Queued,
    Runned,
    Completed,
    Failed,
    Canceled,
}

pub enum JobType {
    OneShot,
    ScheduledAt(DateTime<Utc>),
}

pub enum RetryStrategy {
    Interval(Duration),
    Exponential(Duration),
}

pub struct JobRetry {
    retries: u32,
    max_retries: u32,
    strategy: RetryStrategy,
}

impl JobRetry {
    pub(crate) fn can_retry(&self) -> bool {
        self.retries < self.max_retries
    }

    pub(crate) fn retry_at(&mut self) -> DateTime<Utc> {
        self.retries += 1;
        match &self.strategy {
            RetryStrategy::Interval(t) => Utc::now() + *t,
            RetryStrategy::Exponential(t) => Utc::now() + *t * 2_i32.pow(self.retries),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct JobId(imfield_snowflake::SnowflakeId);

impl From<imfield_snowflake::SnowflakeId> for JobId {
    fn from(id: imfield_snowflake::SnowflakeId) -> Self {
        JobId(id)
    }
}
