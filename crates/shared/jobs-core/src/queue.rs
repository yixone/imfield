use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, MutexGuard},
};

use tokio::{sync::Notify, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::job::JobId;

pub struct JobsQueue {
    state: Mutex<QueueInner>,
    notify: Notify,
}

impl JobsQueue {
    fn lock<'a>(&'a self) -> MutexGuard<'a, QueueInner> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn enqueue(&self, job: JobId) {
        let mut lock = self.lock();
        lock.queued.push_back(job);

        drop(lock);
        self.notify.notify_one();
    }

    pub fn remove_queued(&self, id: JobId) -> bool {
        let mut lock = self.lock();

        let Ok(idx) = lock.queued.binary_search_by(|q| q.cmp(&id)) else {
            return false;
        };

        lock.queued.remove(idx).is_some()
    }

    pub fn clear_queued(&self) -> Vec<JobId> {
        let mut lock = self.lock();

        let ids = lock.queued.iter().copied().collect::<Vec<_>>();
        lock.queued = VecDeque::new();

        ids
    }

    pub async fn pool(&self) -> JobId {
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

    pub fn add_active(&self, active: ActiveJob) {
        let mut lock = self.lock();
        lock.active.insert(active.job, active);
    }

    pub fn get_active(&self, id: JobId) -> Option<ActiveJob> {
        let lock = self.lock();
        lock.active.get(&id).cloned()
    }

    pub fn remove_active(&self, id: JobId) -> Option<ActiveJob> {
        let mut lock = self.lock();
        lock.active.remove(&id)
    }
}

struct QueueInner {
    active: HashMap<JobId, ActiveJob>,
    queued: VecDeque<JobId>,
}

#[derive(Clone)]
pub struct ActiveJob {
    cancel: CancellationToken,
    runned_at: Instant,
    job: JobId,
}

impl ActiveJob {
    pub fn new(job: JobId) -> Self {
        ActiveJob {
            cancel: CancellationToken::new(),
            runned_at: Instant::now(),
            job,
        }
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }
}
