use crate::config::Config;
use crate::{labeler::LabeledMessage, peer::PeerKind};
use log::debug;
use regex::Regex;
use rosc::OscType;
use std::io;
use std::sync::Arc;
use std::{collections::HashMap, sync::OnceLock};

use self::name_lookup::lookup;
use self::names::{filtertypes, fx_types};

mod name_lookup;
mod names;
mod system;

fn address_patterns() -> &'static HashMap<&'static str, Regex> {
    static HASHMAP: OnceLock<HashMap<&'static str, Regex>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(
            "filter_type",
            Regex::new(r"^/param/./filter/./type").expect("Unable to compile regex."),
        );
        m.insert(
            "filter_subtype",
            Regex::new(r"^/param/./filter/./subtype").expect("Unable to compile regex."),
        );
        m.insert(
            "fx_type",
            Regex::new(r"^/param/fx/.*/./type").expect("Unable to compile regex."),
        );

        m
    })
}

/// Inspect messages and route them accordingly. Returns messages after potential alterations.
pub(crate) fn extension_processor(
    config: Arc<Config>,
    mut labeled: LabeledMessage,
) -> Result<LabeledMessage, io::Error> {
    // Handle system messages
    if labeled.message.addr.contains("/sys/") {
        return system::system_handler(config, labeled);
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
    if address_patterns()
        .get("filter_type")
        .unwrap()
        .is_match(labeled.message.addr.as_str())
    {
        return lookup(labeled, filtertypes());
    }

    // Handle fx types
    if address_patterns()
        .get("fx_type")
        .unwrap()
        .is_match(labeled.message.addr.as_str())
    {
        return lookup(labeled, fx_types());
    }

    Ok(labeled)
}
