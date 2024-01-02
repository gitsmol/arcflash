use crate::osc;
use std::{
    io::{self, Error, ErrorKind},
    sync::Arc,
};

use log::debug;
use rand::{thread_rng, Rng};

use crate::peer::Peer;

pub(crate) fn send_message(message: osc::Message, peer_send: Arc<Peer>) -> Result<(), io::Error> {
    // let port = thread_rng().gen_range(8000..50_000);
    let port = 53300;
    let bind_addr = format!("{}:{}", peer_send.local_ip, port);
    debug!(
        "Sending message to {} from {}\n {:?}",
        peer_send, bind_addr, message
    );

    let sender = osc::sender(bind_addr)?;
    match sender.send(message, peer_send.remote_addr()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(
            ErrorKind::Interrupted,
            format!("Error sending message: {}", e.to_string()),
        )),
    }
}
