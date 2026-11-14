use chrono::{DateTime, Utc};

use crate::job::{ExecutableJob, Job, JobId};

pub struct JobsDispatcher<E>
where
    E: ExecutableJob,
{
    context: E::Context,
    flake: imfield_snowflake::SnowflakeGenerator,
}

impl<E> JobsDispatcher<E>
where
    E: ExecutableJob,
{
    pub async fn enqueue(&self, job: E) {
        todo!()
    }

    pub async fn re_enqueue(&self, job: Job<E>) {
        todo!()
    }

    pub async fn remove_queued(&self, id: JobId) {
        todo!()
    }

    pub async fn schedule(&self, job: E, scheduled_at: DateTime<Utc>) {
        todo!()
    }

    pub async fn remove_scheduled(&self, id: JobId) {
        todo!()
    }

    pub async fn mark_job_finished(&self, job: Job<E>) {
        todo!()
    }

    pub async fn mark_job_cancelled(&self, id: JobId) {
        todo!()
    }

    pub async fn mark_job_failed(&self, job: Job<E>) {
        todo!()
    }

    pub async fn next(&self) {
        todo!()
    }
}
