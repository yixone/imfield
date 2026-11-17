use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, MutexGuard},
};

use tokio::sync::Notify;

use crate::job::{ActiveJob, ExecutableJob, InactiveJobStatus, JobId, JobItem};

pub struct JobsQueue<E>
where
    E: ExecutableJob,
{
    state: Mutex<QueueInner<E>>,
    notify: Notify,
}

impl<E> JobsQueue<E>
where
    E: ExecutableJob,
{
    pub(crate) fn lock<'a>(&'a self) -> MutexGuard<'a, QueueInner<E>> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn enqueue(&self, job: JobItem<E>) {
        let mut lock = self.lock();
        lock.queued.push_back(job);

        drop(lock);
        self.notify.notify_one();
    }

    pub fn remove_queued(&self, id: JobId) -> Option<JobItem<E>> {
        let mut lock = self.lock();

        let Ok(idx) = lock.queued.binary_search_by(|q| q.id().cmp(&id)) else {
            return None;
        };

        if let Some(job) = lock.queued.remove(idx) {
            return Some(job);
        }

        None
    }

    pub fn clear_queued(&self) -> usize {
        let mut lock = self.lock();
        lock.queued.drain(..).count()
    }

    pub async fn pool(&self) -> JobItem<E> {
        loop {
            let permit = self.notify.notified();

            if let Some(j) = {
                let mut lock = self.lock();
                lock.queued.pop_front()
            } {
                return j;
            }

            permit.await
        }
    }

    pub fn drain_inactive(&self) -> usize {
        let mut lock = self.lock();
        let len = lock.inactive.len();
        lock.inactive.drain(..=len / 2).count()
    }

    pub fn add_active(&self, job: ActiveJob<E>) {
        let mut lock = self.lock();
        lock.active.insert(job.item().id(), job);
    }

    pub fn remove_active(&self, id: JobId) -> Option<ActiveJob<E>> {
        let mut lock = self.lock();
        lock.active.remove(&id)
    }

    pub fn add_inactive(&self, job: JobItem<E>, status: InactiveJobStatus) {
        let mut lock = self.lock();
        lock.inactive.push((job, status));
    }
}

pub(crate) struct QueueInner<E>
where
    E: ExecutableJob,
{
    pub active: HashMap<JobId, ActiveJob<E>>,
    pub queued: VecDeque<JobItem<E>>,
    pub inactive: Vec<(JobItem<E>, InactiveJobStatus)>,
}
