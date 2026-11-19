use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use crate::job::{ExecutableJob, JobId, JobItem, JobState};

pub struct JobsStorage<E>
where
    E: ExecutableJob,
{
    state: Mutex<HashMap<JobId, JobItem<E>>>,
}

impl<E> JobsStorage<E>
where
    E: ExecutableJob + Clone,
{
    pub fn new() -> Self {
        JobsStorage {
            state: Mutex::new(HashMap::new()),
        }
    }

    fn lock<'a>(&'a self) -> MutexGuard<'a, HashMap<JobId, JobItem<E>>> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn get(&self, id: JobId) -> Option<JobItem<E>> {
        let lock = self.lock();
        lock.get(&id).cloned()
    }

    pub fn save(&self, job: &JobItem<E>) {
        let mut lock = self.lock();
        lock.insert(job.id(), job.clone());
    }

    pub fn try_activate(&self, id: JobId) -> Option<JobItem<E>> {
        let mut lock = self.lock();
        let mut job = lock.get(&id).cloned()?;

        if matches!(
            job.state(),
            JobState::Cancelled | JobState::Active | JobState::Finished
        ) {
            return None;
        }

        job.set_state(JobState::Active);
        lock.insert(job.id(), job.clone());

        Some(job)
    }

    pub fn try_queued(&self, id: JobId) -> bool {
        let mut lock = self.lock();
        let Some(mut job) = lock.get(&id).cloned() else {
            return false;
        };

        if !matches!(job.state(), JobState::Scheduled) {
            return false;
        }

        job.set_state(JobState::Queued);
        lock.insert(job.id(), job.clone());

        true
    }

    pub fn try_cancel(&self, id: JobId) -> bool {
        let mut lock = self.lock();
        let Some(mut job) = lock.get(&id).cloned() else {
            return false;
        };

        job.set_state(JobState::Cancelled);
        lock.insert(job.id(), job.clone());

        true
    }

    pub fn delete(&self, id: JobId) -> Option<JobItem<E>> {
        let mut lock = self.lock();
        lock.remove(&id)
    }

    pub fn cancel_inactive(&self) {
        let mut lock = self.lock();
        for j in lock.values_mut() {
            if matches!(j.state(), JobState::Scheduled | JobState::Queued) {
                j.set_state(JobState::Cancelled);
            }
        }
    }
}
