use serde_value::Value;
use tokio::sync::broadcast;

pub type Sender = broadcast::Sender<Message>;
pub type Receiver = broadcast::Receiver<Message>;

pub fn channel(size: usize) -> (Sender, Receiver) {
    tokio::sync::broadcast::channel(size)
}

#[derive(Debug, Clone)]
pub enum Message {
    Created(String, Value),
    Deleted(String),
}
