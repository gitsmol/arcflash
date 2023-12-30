use std::sync::Arc;

use async_osc::{OscMessage, OscPacket, OscSocket, Result};
use async_recursion::async_recursion;
use async_std::stream::StreamExt;
use log::{debug, info};

use crate::{
    config::Config,
    extension::labeled_message_processor,
    labeler::message_router,
    peer::{Peer, PeerKind},
    sender::{self, send_message},
};

/// Spawns handlers for both peer connections.
pub fn spawn_packet_handlers(config: Arc<Config>) {
    info!("Spawning two UDP handlers.");

    let config_clone = config.clone();

    async_std::task::spawn(async move {
        if let Err(e) = packet_receiver(config.clone(), PeerKind::Instrument).await {
            panic!("Error in osc_handler: {}", e);
        }
    });
    async_std::task::block_on(async move {
        if let Err(e) = packet_receiver(config_clone.clone(), PeerKind::Controller).await {
            panic!("Error in osc_handler: {}", e);
        }
    });
    debug!("Shutting down.");
}

/// Handles incoming OSC packets and routes them to their destination.
async fn packet_receiver(config: Arc<Config>, peer_kind: PeerKind) -> async_osc::Result<()> {
    // Get references to peers from config
    let (peer_recv, peer_send) = match peer_kind {
        PeerKind::Controller => (&config.controller, &config.instrument),
        PeerKind::Instrument => (&config.instrument, &config.controller),
    };

    debug!("Trying to create socket for {}", peer_recv.listen_addr());
    let mut socket = OscSocket::bind(peer_recv.listen_addr()).await?;
    debug!(
        "Listening for {} on {}",
        peer_recv.name,
        peer_recv.listen_addr()
    );

    // Listen for incoming packets on the socket.
    while let Some(packet) = socket.next().await {
        let (message, _) = packet?;
        debug!("Received from {peer_recv}\n {:?}", message);
        unbundler(config.clone(), &peer_recv, &peer_send, message).await?
    }

    Ok(())
}

#[async_recursion]
/// Takes OSC-packets and unbundles them if necessary. Passes unbundled messages on
/// to the labeler for inspection and routing.
async fn unbundler(
    config: Arc<Config>,
    peer_recv: &Peer,
    peer_send: &Peer,
    packet: OscPacket,
) -> Result<()> {
    let message = match packet {
        // If the packet contains a bundle, unbundle and handle individual messages
        OscPacket::Bundle(bundle) => {
            let mut messages: Vec<OscPacket> = Vec::new();
            for packet in bundle.content.iter() {
                messages.push(packet.clone())
            }
            debug!("Packet unbundled, iterating over packets.");
            for message in messages {
                // This recurses so bundles that contain bundles get unpacked ad infinitum
                unbundler(config.clone(), &peer_recv, &peer_send, message).await?;
            }
        }
        // A regular message is passed to the labeler for inspection and routing.
        OscPacket::Message(message) => {
            message_processor(config, peer_recv, peer_send, message).await?
        }
    };

    Ok(())
}

async fn message_processor(
    config: Arc<Config>,
    peer_recv: &Peer,
    peer_send: &Peer,
    message: OscMessage,
) -> Result<()> {
    // If we don't want to use functional extensions, just pass the message on.
    if !config.options.extend {
        send_message(message, peer_send).await?;
        return Ok(());
    }

    let Ok(labeled_message) = message_router(config, peer_recv, peer_send, message) else {
        panic!("Message labeler failed!");
    };
    let Ok(processed_message) = labeled_message_processor(labeled_message) else {
        panic!("Message processor failed!");
    };

    send_message(processed_message.message, processed_message.peer_send).await?;
    Ok(())
}
