use std::sync::Arc;

use crate::{job::ExecutableJob, queue::JobsQueue};

pub struct JobsDispatcher<E>
where
    E: ExecutableJob,
{
    context: E::Context,
    flake: imfield_snowflake::SnowflakeGenerator,

    queue: Arc<JobsQueue<E>>,
}
