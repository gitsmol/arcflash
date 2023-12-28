use std::sync::Arc;

use async_osc::{OscBundle, OscMessage, OscPacket, OscSocket, Result};
use async_recursion::async_recursion;
use async_std::stream::StreamExt;
use log::{debug, info};
use rand::{thread_rng, Rng};

use crate::{extension::extension_filter, peer::Peer};

async fn send_message(peer_recv: &Peer, peer_send: &Peer, message: OscMessage) -> Result<()> {
    debug!(
        "Sending message to {} - {:?}.",
        peer_send.remote_addr(),
        message
    );

    let port = thread_rng().gen_range(8000..50_000);
    let bind_addr = format!("{}:{}", peer_send.local_ip, port);
    // let socket = OscSocket::bind(&appconfig.bind_address).await?;
    debug!("Sending message to {peer_send} using {bind_addr}");

    let socket = OscSocket::bind(bind_addr).await?;
    socket.send_to(message, peer_send.remote_addr()).await?;

    debug!("Sent message to {peer_send}.");

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
async fn osc_unbundle_and_send(
    peer_recv: &Peer,
    peer_send: &Peer,
    packet: OscPacket,
) -> Result<()> {
    match packet {
        OscPacket::Bundle(bundle) => {
            let messages = osc_unbundle(bundle).await?;
            // debug!("Packet unbundled, iterating over packets.");
            for message in messages {
                osc_unbundle_and_send(&peer_recv, &peer_send, message).await?;
            }
        }
        OscPacket::Message(message) => {
            let message_post_filter = extension_filter(message).await?;
            send_message(&peer_recv, &peer_send, message_post_filter).await?;
            debug!("Message sent, waiting for next message.");
        }
    }

    Ok(())
}

async fn osc_handler(peers: Arc<[Peer; 2]>, peer_number: usize) -> async_osc::Result<()> {
    // Create a listening address based on the selected peer
    let peer_recv = peers[peer_number].clone();
    let peer_send = match peer_number {
        0 => peers[1].clone(),
        1 => peers[0].clone(),
        _ => panic!("More than two peers found."),
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
        let (packet, _) = packet?;
        debug!("Received from {peer_recv}: {:?}", packet);
        osc_unbundle_and_send(&peer_recv, &peer_send, packet).await?
    }

    Ok(())
}

pub fn listening_tasks(peers: Arc<[Peer; 2]>) {
    info!("Spawning two UDP handlers.");

    let peers_clone = peers.clone();

    async_std::task::spawn(async move {
        if let Err(e) = osc_handler(peers.clone(), 0).await {
            panic!("Error in osc_handler: {}", e);
        }
    });
    async_std::task::block_on(async move {
        if let Err(e) = osc_handler(peers_clone.clone(), 1).await {
            panic!("Error in osc_handler: {}", e);
        }
    });
    debug!("Shutting down.");
}
