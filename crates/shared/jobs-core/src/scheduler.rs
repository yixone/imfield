use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::{DateTime, Duration, Utc};
use tokio::sync::Notify;

use crate::{
    job::{ExecutableJob, JobItem},
    queue::JobsQueue,
};

pub enum JobSchedule {
    Waited {
        next_run: DateTime<Utc>,
    },
    Interval {
        next_run: DateTime<Utc>,
        interval: Duration,
    },
}

pub struct WaitedJob<E>
where
    E: ExecutableJob,
{
    schedule: JobSchedule,
    job: JobItem<E>,
}

impl<E> WaitedJob<E>
where
    E: ExecutableJob,
{
    pub fn next_run(&self) -> i64 {
        match self.schedule {
            JobSchedule::Waited { next_run } => next_run.timestamp_millis(),
            JobSchedule::Interval { next_run, .. } => next_run.timestamp_millis(),
        }
    }
}

impl<E: ExecutableJob> Ord for WaitedJob<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.next_run().cmp(&self.next_run())
    }
}

impl<E: ExecutableJob> PartialOrd for WaitedJob<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: ExecutableJob> PartialEq for WaitedJob<E> {
    fn eq(&self, other: &Self) -> bool {
        self.job.id() == other.job.id()
    }
}

impl<E: ExecutableJob> Eq for WaitedJob<E> {}

pub struct JobsScheduler<E>
where
    E: ExecutableJob,
{
    queue: Arc<JobsQueue<E>>,
    state: Mutex<SchedulerState<E>>,
    notify: Notify,
}

impl<E> JobsScheduler<E>
where
    E: ExecutableJob,
{
    fn lock<'a>(&'a self) -> MutexGuard<'a, SchedulerState<E>> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
}

struct SchedulerState<E>
where
    E: ExecutableJob,
{
    waited: BinaryHeap<WaitedJob<E>>,
    next: Option<WaitedJob<E>>,
}
