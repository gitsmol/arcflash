use handler::listening_tasks;
use std::sync::Arc;

use log::{debug, info};

use peer::{Peer, PeerKind};

mod extension;
mod handler;
mod peer;

fn main() {
    env_logger::init();

    let controller = Peer {
        name: String::from("TouchOSC"),
        kind: PeerKind::Controller,
        local_ip: String::from("192.168.1.110"),
        remote_ip: String::from("192.168.1.181"),
        remote_port: String::from("53110"),
        local_port: String::from("53100"),
    };
    let instrument = Peer {
        name: String::from("Surge XT"),
        kind: PeerKind::Instrument,
        local_ip: String::from("127.0.0.1"),
        remote_ip: String::from("127.0.0.1"),
        remote_port: String::from("53210"),
        local_port: String::from("53200"),
    };

    let peers = Arc::new([controller, instrument]);

    listening_tasks(peers)
}
