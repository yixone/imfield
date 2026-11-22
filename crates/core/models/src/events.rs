use crate::{entities::asset::AssetStateFailure, types::id::AssetId};

#[derive(Debug)]
pub enum Event {
    AssetProcessingReady {
        id: AssetId,
    },
    AssetProcessingFailed {
        id: AssetId,
        failure: AssetStateFailure,
    },
}

impl Event {
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::AssetProcessingReady { .. } => "asset.processing.ready",
            Event::AssetProcessingFailed { .. } => "asset.processing.failed",
        }
    }
}
