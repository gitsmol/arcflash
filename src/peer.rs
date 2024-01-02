use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Peer {
    pub name: String,       // What is this Peer called?
    pub kind: PeerKind,     // What kind of Peer is this?
    pub local_ip: String,   // What is our address when communicating with this Peer?
    pub local_port: String, // Where do we receive packets from this Peer?

    pub remote_ip: String,   // What is this Peers ip address?
    pub remote_port: String, // Where do send packets to reach this Peer?
}

impl Peer {
    pub(crate) fn remote_addr(&self) -> String {
        format!("{}:{}", self.remote_ip, self.remote_port)
    }

    pub(crate) fn local_addr(&self) -> String {
        format!("{}:{}", self.local_ip, self.local_port)
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) at {}:{}",
            self.name, self.kind, self.remote_ip, self.remote_port
        )
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub(crate) enum PeerKind {
    Controller,
    Instrument,
}

impl Display for PeerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerKind::Controller => write!(f, "Controller"),
            PeerKind::Instrument => write!(f, "Instrument"),
        }
    }
}
