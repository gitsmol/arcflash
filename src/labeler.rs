use crate::{config::Config, peer::Peer};
use async_osc::{OscMessage, Result};
use std::sync::Arc;

#[derive(Clone)]
pub struct LabeledMessage<'a> {
    pub message: OscMessage,
    pub peer_recv: &'a Peer,
    pub peer_send: &'a Peer,
}

pub enum RoutingLabel {
    Passthrough,
    Return,
}

/// The router labels all packages
pub(crate) fn message_router<'a>(
    config: Arc<Config>,
    peer_recv: &'a Peer,
    peer_send: &'a Peer,
    message: OscMessage,
) -> Result<LabeledMessage<'a>> {
    let mut result = LabeledMessage {
        message,
        peer_recv,
        peer_send,
    };

    Ok(result)
}
