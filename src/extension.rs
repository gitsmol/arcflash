use crate::{labeler::LabeledMessage, peer::PeerKind};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use rosc::OscType;
use std::collections::HashMap;
use std::io;

mod filter_type;
mod system;

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
        return system::system_addr(labeled);
    }

    // Handle strings with both real and normalized values
    if labeled.peer_send.kind == PeerKind::Controller {
        if let Some(OscType::String(valstring)) = labeled.message.args.first() {
            if valstring.contains("(normalized)") {
                debug!("Normalized value detected in string.");
                if let Some(float_val) = valstring
                    .split_whitespace()
                    .nth_back(1)
                    .and_then(|s| s.parse::<f32>().ok())
                {
                    if let Some(arg) = labeled.message.args.get_mut(0) {
                        *arg = OscType::Float(float_val);
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
