use std::{sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::{
    job::{ExecutableJob, JobId, JobItem, JobState, JobType},
    queue::{ActiveJob, JobsQueue},
    scheduler::JobsSchedule,
    storage::JobsStorage,
};

pub struct JobsDispatcher<E>
where
    E: ExecutableJob,
{
    context: Arc<E::Context>,
    flake: imfield_snowflake::SnowflakeGenerator,

    queue: Arc<JobsQueue>,
    schedule: Arc<JobsSchedule>,

    storage: Arc<JobsStorage<E>>,

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
            let storage = self.storage.clone();
            let cancel = self.cancel.clone();
            let ctx = self.context.clone();
            let schedule = self.schedule.clone();

            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        job_id = queue.pool() => {
                            let Some(mut job) = storage.try_activate(job_id) else {
                                continue
                            };

                            let active = ActiveJob::new(job_id);
                            queue.add_active(active.clone());

                            match job.execute(&ctx, active.cancel_token()).await {
                                Ok(_) => {
                                    queue.remove_active(job.id());

                                    match job.job_type() {
                                        JobType::Normal | JobType::Schedule { .. } => {
                                            job.set_state(JobState::Finished);

                                            storage.save(&job);
                                        },
                                        JobType::Interval { interval, .. } => {
                                            job.reset_retries();
                                            job.set_state(JobState::Waited);
                                            job.set_job_type(JobType::Interval { next_run_at: Instant::now() + interval, interval });

                                            storage.save(&job);
                                            schedule.schedule(&job);
                                        },
                                    }
                                },
                                Err(e) => {
                                    queue.remove_active(job.id());

                                    if job.retries() >= job.max_retries() {
                                        job.set_state(JobState::Failed(e));
                                        storage.save(&job);
                                        continue;
                                    }

                                    let Some(delay) = job.retry_delay() else {
                                        job.set_state(JobState::Failed(e));
                                        storage.save(&job);
                                        continue;
                                    };

                                    job.set_state(JobState::Waited);
                                    job.add_retry();
                                    job.set_job_type(
                                        match job.job_type() {
                                            JobType::Interval { interval, .. } => JobType::Interval { next_run_at: Instant::now() + delay, interval },
                                            _ => JobType::Schedule { scheduled_at: Instant::now() + delay },
                                        }
                                    );

                                    storage.save(&job);
                                    schedule.schedule(&job);
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
        let storage = self.storage.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job_id) = schedule.next() => {
                        if storage.try_queued(job_id) {
                            queue.enqueue(job_id);
                        }
                    }
                    _ = cancel.cancelled() => {
                        return;
                    }
                }
            }
        })
    }

    pub async fn schedule(&self, job: E, run_at: Instant) -> JobId {
        let mut job = JobItem::new(
            self.flake.get_id_as(),
            JobType::Schedule {
                scheduled_at: run_at,
            },
            job,
        );
        job.set_state(JobState::Waited);

        self.storage.save(&job);
        self.schedule.schedule(&job);

        job.id()
    }

    pub async fn schedule_interval(&self, job: E, run_at: Instant, interval: Duration) -> JobId {
        let mut job = JobItem::new(
            self.flake.get_id_as(),
            JobType::Interval {
                next_run_at: run_at,
                interval,
            },
            job,
        );
        job.set_state(JobState::Waited);

        self.storage.save(&job);
        self.schedule.schedule(&job);

        job.id()
    }

    // --- Queue ---
    pub async fn queue_add(&self, job: E) -> JobId {
        let mut job = JobItem::new(self.flake.get_id_as(), JobType::Normal, job);
        job.set_state(JobState::Queued);

        self.storage.save(&job);
        self.queue.enqueue(job.id());

        job.id()
    }

    // --- State ---
    pub async fn cancel(&self, id: JobId) -> bool {
        // If the task is already running, cancels it
        if let Some(active) = self.queue.get_active(id) {
            active.cancel();
        }

        // Sets the status to "cancelled" in the storage
        self.storage.try_cancel(id)
    }
}
