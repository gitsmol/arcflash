use std::collections::HashMap;

use async_osc::{OscMessage, Result};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ADDR_PATTERNS: HashMap<&'static str, Regex> = {
        let mut m = HashMap::new();
        m.insert(
            "filter_type",
            Regex::new(r"/param/./filter/./type").unwrap(),
        );
        m
    };
}

pub(crate) async fn extension_filter(message: OscMessage) -> Result<OscMessage> {
    let result = match message {
        msg if ADDR_PATTERNS
            .get("filter_type")
            .unwrap()
            .is_match(message.addr.as_str()) =>
        {
            translate_filter_type(msg).await
        }
        _ => message,
    };

    Ok(result)
}

async fn translate_filter_type(message: OscMessage) -> OscMessage {
    message
}

async fn message_translator(message: OscMessage) -> OscMessage {
    message
}
