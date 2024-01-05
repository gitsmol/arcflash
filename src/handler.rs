use log::{debug, info, warn};

use crate::{
    extension::extension_processor,
    labeler::LabeledMessage,
    osc::{self, *},
    peer::{Peer, PeerKind},
    sender::send_message,
};
use std::{io, sync::Arc, thread::JoinHandle};

use crate::config::Config;

pub fn spawn_handler(config: Arc<Config>, peer_kind: PeerKind) -> JoinHandle<()> {
    let (peer_recv, peer_send) = match peer_kind {
        PeerKind::Controller => (
            Arc::new(config.controller.clone()),
            Arc::new(config.instrument.clone()),
        ),
        PeerKind::Instrument => (
            Arc::new(config.instrument.clone()),
            Arc::new(config.controller.clone()),
        ),
    };

    let mut packages_received: usize = 0;

    std::thread::spawn(move || {
        let recv_local =
            receiver(peer_recv.local_addr(), 1024).expect("Failed to bind receiver to local ip.");
        info!("Receiver thread starting for {}", peer_recv.local_addr());

        while let Ok((packet, _)) = recv_local.recv() {
            match packet_sorter(config.clone(), peer_recv.clone(), peer_send.clone(), packet) {
                Ok(_) => {}
                Err(e) => warn!("Error handling packet: {}", e),
            };
            packages_received += 1;
            info!(
                "Packages received on {} thread: {}",
                peer_kind, packages_received
            );
        }
    })
}

fn packet_sorter(
    config: Arc<Config>,
    peer_recv: Arc<Peer>,
    peer_send: Arc<Peer>,
    packet: Packet,
) -> Result<(), io::Error> {
    for message in packet.into_msgs() {
        // If we don't want to use functional extensions, just pass the message on.
        match config.options.extend {
            true => message_processor(peer_recv.clone(), peer_send.clone(), message)?,
            false => send_message(message, peer_send.clone())?,
        }
    }

    Ok(())
}

fn message_processor(
    peer_recv: Arc<Peer>,
    peer_send: Arc<Peer>,
    message: osc::Message,
) -> Result<(), io::Error> {
    let labeled_message = LabeledMessage::new(peer_recv, peer_send, message);

    let processed_message = extension_processor(labeled_message)
        .map_err(|e| io::Error::new(e.kind(), format!("Error in extension processor: {}", e)))?;

    send_message(processed_message.message, processed_message.peer_send)
        .map_err(|e| io::Error::new(e.kind(), format!("Error sending message: {}", e)))?;

    Ok(())
}
