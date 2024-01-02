use std::sync::Arc;

use crate::osc::Message;
use crate::peer::Peer;

#[derive(Clone)]
pub struct LabeledMessage {
    pub message: Message,
    pub peer_recv: Arc<Peer>,
    pub peer_send: Arc<Peer>,
}

impl LabeledMessage {
    pub fn new(peer_recv: Arc<Peer>, peer_send: Arc<Peer>, message: Message) -> Self {
        Self {
            message,
            peer_recv,
            peer_send,
        }
    }
}
