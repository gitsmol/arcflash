use crate::{
    labeler::{message_router, LabeledMessage, RoutingLabel},
    peer::{Peer, PeerKind},
};
use async_osc::{prelude::OscMessageExt, OscMessage, Result};
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

pub(crate) fn labeled_message_processor<'a>(
    mut labeled: LabeledMessage<'a>,
) -> Result<LabeledMessage> {
    if labeled.message.addr.contains("/sys/") {
        return system_addr(labeled);
    }

    if ADDR_PATTERNS
        .get("filter_type")
        .unwrap()
        .is_match(labeled.message.addr.as_str())
    {
        return filter_type::translate_filter_type(labeled);
    }

    Ok(labeled)
}

fn system_addr(mut labeled: LabeledMessage) -> Result<LabeledMessage> {
    labeled.routing = RoutingLabel::Return;
    Ok(labeled)
}
