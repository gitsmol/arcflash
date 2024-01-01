use std::sync::Arc;

use crate::peer::Peer;
use async_osc::OscMessage;

#[derive(Clone)]
pub struct LabeledMessage {
    pub message: OscMessage,
    pub peer_recv: Arc<Peer>,
    pub peer_send: Arc<Peer>,
}

impl LabeledMessage {
    pub fn new(peer_recv: Arc<Peer>, peer_send: Arc<Peer>, message: OscMessage) -> Self {
        Self {
            message,
            peer_recv,
            peer_send,
        }
    }
}
