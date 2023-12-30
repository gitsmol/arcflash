use async_osc::{OscMessage, OscSocket, Result};
use log::debug;
use rand::{thread_rng, Rng};

use crate::{labeler::LabeledMessage, peer::Peer};

pub(crate) async fn send_message<'a>(labeled: LabeledMessage<'a>) -> Result<()> {
    let port = thread_rng().gen_range(8000..50_000);
    let bind_addr = format!("{}:{}", labeled.peer_send.local_ip, port);
    debug!(
        "Sending message to {} from {}\n {:?}",
        labeled.peer_send, bind_addr, labeled.message
    );

    let socket = OscSocket::bind(bind_addr).await?;
    socket
        .send_to(labeled.message, labeled.peer_send.remote_addr())
        .await?;

    Ok(())
}
