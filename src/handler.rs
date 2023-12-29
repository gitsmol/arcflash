use std::sync::Arc;

use async_osc::{OscBundle, OscMessage, OscPacket, OscSocket, Result};
use async_recursion::async_recursion;
use async_std::stream::StreamExt;
use log::{debug, info};
use rand::{thread_rng, Rng};

use crate::{
    config::Config,
    extension::extension_filter,
    peer::{Peer, PeerKind},
};

async fn send_message(peer_send: &Peer, message: OscMessage) -> Result<()> {
    let port = thread_rng().gen_range(8000..50_000);
    let bind_addr = format!("{}:{}", peer_send.local_ip, port);
    debug!(
        "Sending message to {peer_send} from {bind_addr}\n {:?}",
        message
    );

    let socket = OscSocket::bind(bind_addr).await?;
    socket.send_to(message, peer_send.remote_addr()).await?;

    Ok(())
}

pub(crate) async fn osc_unbundle(bundle: OscBundle) -> Result<Vec<OscPacket>> {
    let mut result: Vec<OscPacket> = Vec::new();
    for packet in bundle.content.iter() {
        result.push(packet.clone())
    }

    Ok(result)
}

#[async_recursion]
/// Takes OSC-packets and unbundles them if necessary.
/// If the 'extend' option is true, messages go through `extension::extension_filter`.
async fn osc_unbundle_and_send(
    config: Arc<Config>,
    peer_recv: &Peer,
    peer_send: &Peer,
    packet: OscPacket,
) -> Result<()> {
    match packet {
        OscPacket::Bundle(bundle) => {
            let messages = osc_unbundle(bundle).await?;
            debug!("Packet unbundled, iterating over packets.");
            for message in messages {
                osc_unbundle_and_send(config.clone(), &peer_recv, &peer_send, message).await?;
            }
        }
        OscPacket::Message(message) => {
            let message_to_send = match config.options.extend {
                true => extension_filter(peer_recv, peer_send, message).await?,
                false => message,
            };
            send_message(&peer_send, message_to_send).await?;
        }
    }

    Ok(())
}

/// Handles incoming OSC packets and routes them to their destination.
async fn osc_handler(config: Arc<Config>, peer_kind: PeerKind) -> async_osc::Result<()> {
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
        osc_unbundle_and_send(config.clone(), &peer_recv, &peer_send, message).await?
    }

    Ok(())
}

/// Spawns handlers for both peer connections.
pub fn listening_tasks(config: Arc<Config>) {
    info!("Spawning two UDP handlers.");

    let config_clone = config.clone();

    async_std::task::spawn(async move {
        if let Err(e) = osc_handler(config.clone(), PeerKind::Instrument).await {
            panic!("Error in osc_handler: {}", e);
        }
    });
    async_std::task::block_on(async move {
        if let Err(e) = osc_handler(config_clone.clone(), PeerKind::Controller).await {
            panic!("Error in osc_handler: {}", e);
        }
    });
    debug!("Shutting down.");
}
