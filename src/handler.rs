use std::{
    collections::VecDeque,
    ops::Deref,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{self},
    time::Duration,
};

use async_osc::{OscMessage, OscPacket, OscSocket, Result};
use async_recursion::async_recursion;
use async_std::stream::{StreamExt, Timeout};

use log::{debug, warn};

use crate::{
    config::Config,
    extension::labeled_message_processor,
    labeler::LabeledMessage,
    peer::{Peer, PeerKind},
    sender::send_message,
};

pub fn thread_peer_handler(config: Arc<Config>, peer_kind: PeerKind) -> thread::JoinHandle<()> {
    // Clone peers into threads
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

    thread::spawn(move || {
        // Get references to peers from config

        let (buffer_tx, buffer_rx) = channel::<OscPacket>();

        let t1 = thread_packet_handler(config.clone(), peer_recv.clone(), peer_send, buffer_rx);

        async_std::task::spawn(async move {
            debug!(
                "Listening for {} on {}",
                peer_recv.name,
                peer_recv.listen_addr()
            );
            if let Err(e) = packet_inbound_buffer(peer_recv.listen_addr(), buffer_tx).await {
                panic!("Error in osc_handler: {}", e);
            }
        });

        while !t1.is_finished() {}
    })
}

fn thread_packet_handler(
    config: Arc<Config>,
    peer_recv: Arc<Peer>,
    peer_send: Arc<Peer>,
    buffer_rx: Receiver<OscPacket>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        debug!("Spawned package handler thread.");
        let mut buffer: VecDeque<OscPacket> = VecDeque::new();

        loop {
            if let Ok(packet) = buffer_rx.try_recv() {
                buffer.push_back(packet);
                debug!("Buffer size: {}", buffer.len());
            } else {
                if let Some(packet) = buffer.pop_front() {
                    debug!("Packet taken from buffer: {:?}", packet);
                    let config = config.clone();
                    let peer_recv = peer_recv.clone();
                    let peer_send = peer_send.clone();
                    async_std::task::spawn(async move {
                        unbundler(config, peer_recv, peer_send, packet).await
                    });
                    debug!("Buffer size: {}", buffer.len());
                }
            };
        }
    })
}

/// Handles incoming OSC packets and routes them to their destination.
async fn packet_inbound_buffer(
    listen_addr: String,
    buffer_tx: Sender<OscPacket>,
) -> async_osc::Result<()> {
    let mut socket = OscSocket::bind(listen_addr).await?;

    // Listen for incoming packets on the socket.
    while let Some(Ok((packet, _))) = socket.next().await {
        debug!("Received packet: {:?}", packet);
        if buffer_tx.send(packet).is_err() {
            warn!("Failed to send labeled packet to handler thread.");
        };
    }

    Ok(())
}

// #[async_recursion]
/// Takes OSC-packets and unbundles them if necessary. Passes unbundled messages on
/// to the labeler for inspection and routing.
async fn unbundler(
    config: Arc<Config>,
    peer_recv: Arc<Peer>,
    peer_send: Arc<Peer>,
    packet: OscPacket,
) -> Result<()> {
    match packet {
        // If the packet contains a bundle, unbundle and handle individual messages
        OscPacket::Bundle(bundle) => {
            let mut messages: Vec<OscPacket> = Vec::new();
            for packet in bundle.content.iter() {
                messages.push(packet.clone())
            }
            debug!("Packet unbundled, iterating over packets.");
            for message in messages {
                // This recurses so bundles that contain bundles get unpacked ad infinitum
                unbundler(
                    config.clone(),
                    peer_recv.clone(),
                    peer_send.clone(),
                    message,
                );
            }
        }
        // A regular message is passed to the labeler for inspection and routing.
        OscPacket::Message(message) => {
            // If we don't want to use functional extensions, just pass the message on.
            match config.options.extend {
                true => message_processor(peer_recv, peer_send, message)?,
                false => {
                    async_std::task::spawn(async move {
                        send_message(message, peer_send).await;
                    });
                }
            };
        }
    };

    Ok(())
}

fn message_processor(
    peer_recv: Arc<Peer>,
    peer_send: Arc<Peer>,
    message: OscMessage,
) -> Result<()> {
    let labeled_message = LabeledMessage::new(peer_recv, peer_send, message);

    let Ok(processed_message) = labeled_message_processor(labeled_message) else {
        panic!("Message processor failed!");
    };

    async_std::task::spawn(async move {
        send_message(processed_message.message, processed_message.peer_send).await;
    });
    Ok(())
}
