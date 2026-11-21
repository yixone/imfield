use std::{sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::{
    job::{ExecutableJob, JobId, JobItem, JobState, JobType},
    queue::{ActiveJob, JobsQueue},
    scheduler::JobsSchedule,
    storage::JobsStorage,
};

/// A lightweight in-process dispatcher for background jobs
///
/// `JobsDispatcher` manages job execution using a fixed number of workers,
/// an in-memory queue and a scheduler for delayed and recurring jobs
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
    /// Creates a new job dispatcher
    ///
    /// The dispatcher is initialized with an execution context, an ID
    /// generator and a fixed number of workers
    ///
    /// # Panics
    ///
    /// Panics if `workers = 0`
    pub fn new(
        context: Arc<E::Context>,
        flake: imfield_snowflake::SnowflakeGenerator,
        workers: usize,
    ) -> Self {
        assert_ne!(workers, 0);

        JobsDispatcher {
            context,
            flake,
            queue: Arc::new(JobsQueue::new()),
            schedule: Arc::new(JobsSchedule::new()),
            storage: Arc::new(JobsStorage::new()),
            cancel: CancellationToken::new(),
            workers,
        }
    }

    /// Schedules a job for execution at the specified time
    ///
    /// The job is added to the scheduler and becomes eligible for
    /// execution when `run_at` is reached
    ///
    /// Returns the ID assigned to the job
    pub fn schedule(&self, job: E, run_at: Instant) -> JobId {
        let mut job = JobItem::new(
            self.flake.get_id_as(),
            JobType::Schedule {
                scheduled_at: run_at,
            },
            job,
        );
        job.set_state(JobState::Scheduled);

        self.storage.save(&job);
        self.schedule.schedule(&job);

        job.id()
    }

    /// Schedules a recurring job
    ///
    /// The first execution is scheduled for `run_at`. After each successful
    /// execution, the next run is scheduled after `interval` has elapsed
    ///
    /// Returns the ID assigned to the job
    pub fn schedule_interval(&self, job: E, run_at: Instant, interval: Duration) -> JobId {
        let mut job = JobItem::new(
            self.flake.get_id_as(),
            JobType::Interval {
                next_run_at: run_at,
                interval,
            },
            job,
        );
        job.set_state(JobState::Scheduled);

        self.storage.save(&job);
        self.schedule.schedule(&job);

        job.id()
    }

    /// Adds a job to the queue for immediate execution
    ///
    /// The job becomes available to workers as soon as it is enqueued
    ///
    /// Returns the ID assigned to the job
    pub fn queue_add(&self, job: E) -> JobId {
        let mut job = JobItem::new(self.flake.get_id_as(), JobType::Normal, job);
        job.set_state(JobState::Queued);

        self.storage.save(&job);
        self.queue.enqueue(job.id());

        job.id()
    }

    /// Cancels all jobs that are not currently executing
    ///
    /// Queued jobs are removed from the execution queue, and inactive jobs
    /// are marked as cancelled in the sotrage
    ///
    /// Currenly executing jobs are not affected by this operation
    pub fn cancel_inactive(&self) {
        self.queue.clear_queued();
        self.storage.cancel_inactive();
    }

    /// Attempts to cancel a job by its ID
    ///
    /// If the job is currently executing, its cancellation token is
    /// signalled. Otherwise, the dispatcher attempts to mark the job
    /// as cancelled in the storage
    pub fn cancel(&self, id: JobId) -> bool {
        // If the task is already running, cancels it
        if let Some(active) = self.queue.get_active(id) {
            active.cancel();
        }

        // Sets the status to "cancelled" in the storage
        self.storage.try_cancel(id)
    }

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

                            let result = job.execute(&ctx, active.cancel_token()).await;

                            queue.remove_active(job.id());

                            // Handling a cancelled job
                            if storage.is_cancelled(job.id()) {
                                continue;
                            }

                            // Processing the result of a job execution
                            match result {
                                Ok(_) => {
                                    match job.job_type() {
                                        JobType::Normal | JobType::Schedule { .. } => {
                                            job.set_state(JobState::Finished);

                                            storage.save(&job);
                                        },
                                        JobType::Interval { interval, .. } => {
                                            job.reset_retries();
                                            job.set_state(JobState::Scheduled);
                                            job.set_job_type(JobType::Interval { next_run_at: Instant::now() + interval, interval });

                                            storage.save(&job);
                                            schedule.schedule(&job);
                                        },
                                    }
                                },
                                Err(e) => {
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

                                    job.set_state(JobState::Scheduled);
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
}

/// Starts the job dispatcher
///
/// Spawns one scheduler task and the configured number of worker tasks
/// on the current [`tokio`] runtime
///
/// Returns the join handles of all spawned tasks
pub fn run_dispatcher<E>(dispatcher: &JobsDispatcher<E>) -> Vec<JoinHandle<()>>
where
    E: ExecutableJob + Clone + 'static,
{
    let mut handles = Vec::with_capacity(dispatcher.workers + 1);

    handles.push(dispatcher.scheduler_loop());
    handles.extend(dispatcher.workers_loop());

    handles
}
