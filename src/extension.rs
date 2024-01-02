use crate::osc;
use crate::{labeler::LabeledMessage, peer::PeerKind};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::collections::HashMap;
use std::io;

mod filter_type;

lazy_static! {
    static ref ADDR_PATTERNS: HashMap<&'static str, Regex> = {
        let mut m = HashMap::new();
        m.insert(
            "filter_type",
            Regex::new(r"^/param/./filter/./type").expect("Unable to compile regex."),
        );
        m
    };
}

/// Inspect messages and route them accordingly. Returns messages after potential alterations.
pub(crate) fn extension_processor(
    mut labeled: LabeledMessage,
) -> Result<LabeledMessage, io::Error> {
    // Handle system messages
    if labeled.message.addr.contains("/sys/") {
        return system_addr(labeled);
    }

    // Handle strings with both real and normalized values
    if labeled.peer_send.kind == PeerKind::Controller {
        if let Some(osc::Type::String(valstring)) = labeled.message.args.first() {
            if valstring.contains("(normalized)") {
                debug!("Normalized value detected in string.");
                if let Some(float_val) = valstring
                    .split_whitespace()
                    .nth_back(1)
                    .and_then(|s| s.parse::<f32>().ok())
                {
                    if let Some(arg) = labeled.message.args.get_mut(0) {
                        *arg = osc::Type::Float(float_val);
                    }
                }
            }
        }
    }

    // Handle filter types
    if ADDR_PATTERNS
        .get("filter_type")
        .unwrap()
        .is_match(labeled.message.addr.as_str())
    {
        return filter_type::translate_filter_type(labeled);
    }

    Ok(labeled)
}

/// System messages are addressed to Arcflash i.e. the packet router.
/// If the packet router runs on the system the instrument runs on, then system
/// diagnostics from the router can be used to indicate the instruments health too.
/// System messages are always returned to the the peer they were received from.
fn system_addr(labeled: LabeledMessage) -> Result<LabeledMessage, io::Error> {
    debug!("Received system message from {}.", labeled.peer_recv);

    // System average load
    if labeled.message.addr.contains("/sys/q/system_load") {
        if let Ok(load) = sys_info::loadavg() {
            let load_message = format!(
                "1 min: {:.2}, 5 min: {:.2}, 15 min: {:.2}",
                load.one, load.five, load.fifteen
            );
            let addr = String::from("/sys/system_load");
            let return_message = build_return_message_string(labeled, addr, load_message);
            return Ok(return_message);
        }
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

fn build_return_message_string(
    labeled: LabeledMessage,
    addr: String,
    content: String,
) -> LabeledMessage {
    let mut return_message = labeled.clone();
    return_message.peer_send = return_message.peer_recv.clone();
    return_message.message = osc::Message {
        addr,
        args: vec![osc::Type::String(content)],
    };
    return_message
}
