use std::io;

use log::debug;
use rosc::OscType;

use crate::{labeler::LabeledMessage, osc};

/// System messages are addressed to Arcflash i.e. the packet router.
/// If the packet router runs on the system the instrument runs on, then system
/// diagnostics from the router can be used to indicate the instruments health too.
/// System messages are always returned to the the peer they were received from.
pub fn system_addr(labeled: LabeledMessage) -> Result<LabeledMessage, io::Error> {
    debug!("Received system message from {}.", labeled.peer_recv);

    // System average load
    if labeled.message.addr.contains("/sys/q/system_load") {
        if let Ok(load) = sys_info::loadavg() {
            let load_message = OscType::String(format!(
                "1 min: {:.2}, 5 min: {:.2}, 15 min: {:.2}",
                load.one, load.five, load.fifteen
            ));
            let addr = String::from("/sys/system_load");
            let return_message = build_return_message(labeled, addr, load_message);
            return Ok(return_message);
        }
    }

    // System average load
    if labeled.message.addr.contains("/sys/q/arcflash") {
        let load_message = OscType::Bool(true);
        let addr = String::from("/sys/arcflash");
        let return_message = build_return_message(labeled, addr, load_message);
        return Ok(return_message);
    }

    // If we can't match any addresses, return a not found message.
    debug!("Unable to match system message to address.");
    let return_message = LabeledMessage {
        message: osc::Message {
            addr: String::from("/sys/debug"),
            args: vec![osc::Type::String(String::from("Unknown address."))],
        },

        peer_recv: labeled.peer_recv.clone(),
        peer_send: labeled.peer_recv.clone(),
    };
    Ok(return_message)
}

fn build_return_message(labeled: LabeledMessage, addr: String, content: OscType) -> LabeledMessage {
    let mut return_message = labeled.clone();
    return_message.peer_send = return_message.peer_recv.clone();
    return_message.message = osc::Message {
        addr,
        args: vec![content],
    };
    return_message
}
