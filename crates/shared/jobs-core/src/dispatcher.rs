use std::{sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::{
    job::{ActiveJob, ExecutableJob, InactiveJobStatus, JobId, JobItem, JobType},
    queue::JobsQueue,
    scheduler::JobsSchedule,
};

pub struct JobsDispatcher<E>
where
    E: ExecutableJob,
{
    context: Arc<E::Context>,
    flake: imfield_snowflake::SnowflakeGenerator,

    queue: Arc<JobsQueue<E>>,
    schedule: Arc<JobsSchedule<E>>,

    cancel: CancellationToken,

    workers: usize,
}

impl<E> JobsDispatcher<E>
where
    E: ExecutableJob + Clone + 'static,
{
    // --- Workers ---
    fn workers_loop(&self) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::with_capacity(self.workers);

        for _ in 0..self.workers {
            let queue = self.queue.clone();
            let cancel = self.cancel.clone();
            let ctx = self.context.clone();
            let schedule = self.schedule.clone();

            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        job = queue.pool() => {
                            let active = ActiveJob::new(job);
                            queue.add_active(active.clone());


                            match active.job.execute(&ctx).await {
                                Ok(_) => {
                                    let mut job = active.job;
                                    queue.remove_active(job.id());

                                    match job.job_type() {
                                        JobType::Normal | JobType::Schedule { .. } => {
                                            queue.add_inactive(job, InactiveJobStatus::Finished);
                                        },
                                        JobType::Interval { interval, .. } => {
                                            job.reset_retries();
                                            job.set_job_type(JobType::Interval { next_run_at: Instant::now() + interval, interval });
                                            schedule.schedule(job);
                                        },
                                    }
                                },
                                Err(e) => {
                                    let mut job = active.job;
                                    queue.remove_active(job.id());

                                    if job.retries() >= job.max_retries() {
                                        queue.add_inactive(job, InactiveJobStatus::Failed(e));
                                        continue;
                                    }

                                    let Some(delay) = job.retry_delay() else {
                                        queue.add_inactive(job, InactiveJobStatus::Failed(e));
                                        continue;
                                    };

                                    job.add_retry();
                                    job.set_job_type(
                                        match job.job_type() {
                                            JobType::Interval { interval, .. } => JobType::Interval { next_run_at: Instant::now() + delay, interval },
                                            _ => JobType::Schedule { scheduled_at: Instant::now() + delay },
                                        }
                                    );

                                    schedule.schedule(job);
                                },
                            }
                        }
                        _ = cancel.cancelled() => return,
                    }
                }
            });
            handles.push(handle);
        }

        handles
    }

    // --- Schedule ---
    fn scheduler_loop(&self) -> JoinHandle<()> {
        let queue = self.queue.clone();
        let schedule = self.schedule.clone();
        let cancel = self.cancel.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = schedule.next() => {
                        queue.enqueue(job);
                    }
                    _ = cancel.cancelled() => {
                        return;
                    }
                }
            }
        })
    }

    pub async fn schedule(&self, job: E, run_at: Instant) -> JobId {
        let id = self.flake.get_id_as();
        let job = JobItem::new(
            id,
            JobType::Schedule {
                scheduled_at: run_at,
            },
            job,
        );

        self.schedule.schedule(job);

        id
    }

    pub async fn schedule_interval(&self, job: E, run_at: Instant, interval: Duration) -> JobId {
        let id = self.flake.get_id_as();
        let job = JobItem::new(
            id,
            JobType::Interval {
                next_run_at: run_at,
                interval,
            },
            job,
        );

        self.schedule.schedule(job);

        id
    }

    pub async fn remove_scheduled(&self, id: JobId) {
        self.schedule.remove_scheduled(id);
    }

    // --- Queue ---
    pub async fn enqueue(&self, job: E) -> JobId {
        let id = self.flake.get_id_as();
        let job = JobItem::new(id, JobType::Normal, job);

        self.queue.enqueue(job);

        id
    }

    pub async fn remove(&self, id: JobId) -> bool {
        if let Some(active) = self.queue.remove_active(id) {
            active.cancel();
            return true;
        }

        self.queue.remove_queued(id).is_some()
    }

    pub async fn clear_queue(&self) -> usize {
        self.queue.clear_queued()
    }
}
