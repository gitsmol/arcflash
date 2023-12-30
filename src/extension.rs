use crate::labeler::LabeledMessage;
use async_osc::{prelude::OscMessageExt, OscMessage, OscType, Result};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::collections::HashMap;

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
pub(crate) fn labeled_message_processor<'a>(
    mut labeled: LabeledMessage<'a>,
) -> Result<LabeledMessage> {
    // Handle system messages
    if labeled.message.addr.contains("/sys/") {
        return system_addr(labeled);
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
fn system_addr(labeled: LabeledMessage) -> Result<LabeledMessage> {
    debug!("Received system message from {}.", labeled.peer_recv);
    if labeled.message.addr.contains("/sys/q/system_load") {
        if let Ok(load) = sys_info::loadavg() {
            let load_message = format!(
                "1 min: {:.2}, 5 min: {:.2}, 15 min: {:.2}",
                load.one, load.five, load.fifteen
            );
            let mut return_message = labeled.clone();
            return_message.peer_send = return_message.peer_recv;
            return_message.message =
                OscMessage::new("/sys/system_load", OscType::String(load_message));

            debug!("Returning system load to {}.", return_message.peer_send);
            return Ok(return_message);
        }
    }

    // If we can't match any addresses, return a not found message.
    debug!("Unable to match system message to address.");
    let return_message = LabeledMessage {
        message: OscMessage::new(
            "/sys/debug",
            OscType::String(String::from("Unknown address.")),
        ),
        peer_recv: labeled.peer_recv,
        peer_send: labeled.peer_recv,
    };
    Ok(return_message)
}
