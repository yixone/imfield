use std::{
    collections::BinaryHeap,
    sync::{Mutex, MutexGuard},
};

use tokio::{sync::Notify, time::Instant};

use crate::job::{ExecutableJob, JobId, JobItem, JobType};

pub struct JobsSchedule {
    state: Mutex<ScheduleState>,
    notify: Notify,
}

impl JobsSchedule {
    pub fn new() -> Self {
        JobsSchedule {
            state: Mutex::new(ScheduleState {
                waited: BinaryHeap::new(),
                next: None,
            }),
            notify: Notify::new(),
        }
    }

    fn lock<'a>(&'a self) -> MutexGuard<'a, ScheduleState> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub async fn next(&self) -> Option<JobId> {
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

    pub fn schedule<E>(&self, job: &JobItem<E>)
    where
        E: ExecutableJob,
    {
        let next_run = match job.job_type() {
            JobType::Normal => Instant::now(),
            JobType::Schedule { scheduled_at } => scheduled_at,
            JobType::Interval { next_run_at, .. } => next_run_at,
        };

        let waited = WaitedJob {
            next_run,
            job: job.id(),
        };

        let mut lock = self.lock();
        lock.waited.push(waited);

        drop(lock);
        self.notify.notify_one();
    }

    pub fn remove_scheduled(&self, id: JobId) {
        let mut lock = self.lock();

        if let Some(n) = &lock.next
            && n.job == id
        {
            lock.next = None;
            drop(lock);

            self.notify.notify_one();
            return;
        }

        lock.waited.retain(|j| j.job != id);

        drop(lock);
        self.notify.notify_one();
    }

    pub fn take_next(&self) -> Option<WaitedJob> {
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
}

struct ScheduleState {
    waited: BinaryHeap<WaitedJob>,
    next: Option<WaitedJob>,
}

#[derive(Clone)]
pub struct WaitedJob {
    next_run: Instant,
    job: JobId,
}

impl Ord for WaitedJob {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.next_run.cmp(&self.next_run)
    }
}

impl PartialOrd for WaitedJob {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WaitedJob {
    fn eq(&self, other: &Self) -> bool {
        self.job == other.job
    }
}

impl Eq for WaitedJob {}
