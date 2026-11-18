use std::{
    collections::BinaryHeap,
    sync::{Mutex, MutexGuard},
};

use tokio::{sync::Notify, time::Instant};

use crate::job::{ExecutableJob, JobId, JobItem, JobType};

pub struct JobsSchedule<E>
where
    E: ExecutableJob,
{
    state: Mutex<ScheduleState<E>>,
    notify: Notify,
}

impl<E> JobsSchedule<E>
where
    E: ExecutableJob + Clone,
{
    fn lock<'a>(&'a self) -> MutexGuard<'a, ScheduleState<E>> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub async fn next(&self) -> Option<JobItem<E>> {
        loop {
            let next = self.take_next()?;

            tokio::select! {
                _ = self.notify.notified() => {
                    self.restore_next();
                    continue;
                }
                _ = tokio::time::sleep_until(next.next_run) => {
                    return Some(next.job)
                }
            }
        }
    }

    pub fn schedule(&self, job: JobItem<E>) {
        let next_run = match job.job_type() {
            JobType::Normal => Instant::now(),
            JobType::Schedule { scheduled_at } => scheduled_at,
            JobType::Interval { next_run_at, .. } => next_run_at,
        };
        let waited = WaitedJob { next_run, job };

        let mut lock = self.lock();
        lock.waited.push(waited);

        drop(lock);
        self.notify.notify_one();
    }

    pub fn remove_scheduled(&self, id: JobId) {
        let mut lock = self.lock();

        if let Some(n) = &lock.next
            && n.job.id() == id
        {
            lock.next = None;
            drop(lock);

            self.notify.notify_one();
            return;
        }

        lock.waited.retain(|j| j.job.id() != id);

        drop(lock);
        self.notify.notify_one();
    }

    pub fn take_next(&self) -> Option<WaitedJob<E>> {
        let mut lock = self.lock();

        let next = lock.waited.pop()?;
        lock.next = Some(next.clone());

        Some(next)
    }

    pub fn restore_next(&self) {
        let mut lock = self.lock();

        if let Some(scheduled) = lock.next.take() {
            lock.waited.push(scheduled);
            lock.next = None;
        }
    }

    pub fn notify(&self) -> &Notify {
        &self.notify
    }
}

struct ScheduleState<E>
where
    E: ExecutableJob,
{
    waited: BinaryHeap<WaitedJob<E>>,
    next: Option<WaitedJob<E>>,
}

#[derive(Clone)]
pub struct WaitedJob<E>
where
    E: ExecutableJob,
{
    next_run: Instant,
    job: JobItem<E>,
}

impl<E: ExecutableJob> Ord for WaitedJob<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.next_run.cmp(&self.next_run)
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
