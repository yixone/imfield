use std::sync::Arc;

use tokio::sync::broadcast;

pub struct EventBus<E> {
    pub(crate) sender: broadcast::Sender<Arc<E>>,
}

impl<E> EventBus<E> {
    pub fn new(channel_size: usize) -> EventBus<E> {
        let sender = broadcast::Sender::new(channel_size);
        EventBus { sender }
    }

    pub fn stream(&self) -> EventStream<E> {
        let receiver = self.sender.subscribe();
        EventStream { receiver }
    }

    pub fn publish(&self, event: E) -> usize {
        self.sender.send(Arc::new(event)).unwrap_or_default()
    }
}

pub struct EventStream<E> {
    pub(crate) receiver: broadcast::Receiver<Arc<E>>,
}

impl<E> EventStream<E> {
    pub async fn recv(&mut self) -> Result<Arc<E>, EventStreamError> {
        self.receiver.recv().await.map_err(EventStreamError::from)
    }
}

#[derive(Debug)]
pub enum EventStreamError {
    ChannelClosed,
    Lagged(u64),
}

impl std::fmt::Display for EventStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for EventStreamError {}

impl From<broadcast::error::RecvError> for EventStreamError {
    fn from(err: broadcast::error::RecvError) -> Self {
        match err {
            broadcast::error::RecvError::Closed => EventStreamError::ChannelClosed,
            broadcast::error::RecvError::Lagged(l) => EventStreamError::Lagged(l),
        }
    }
}
