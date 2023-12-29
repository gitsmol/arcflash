use std::collections::HashMap;

use async_osc::{prelude::OscMessageExt, OscMessage, Result};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;

use crate::peer::{Peer, PeerKind};

lazy_static! {
    static ref ADDR_PATTERNS: HashMap<&'static str, Regex> = {
        let mut m = HashMap::new();
        m.insert(
            "filter_type",
            Regex::new(r"^/param/./filter/./type").expect("Unable to compile regex."),
        );
        m
    };
    static ref FILTERTYPES: HashMap<i32, String> = HashMap::from([
        (0, String::from("Off")),
        (1, String::from("LP 12 dB")),
        (2, String::from("LP 24 dB")),
        (3, String::from("LP Legacy Ladder")),
        (10, String::from("LP Vintage Ladder")),
        (13, String::from("LP K35")),
        (15, String::from("LP Diode Ladder")),
        (11, String::from("LP OB-Xd 12 dB")),
        (12, String::from("LP OB-Xd 24 dB")),
        (16, String::from("LP Cutoff Warp")),
        (28, String::from("LP Res Warp")),
        (6, String::from("BP 12 dB")),
        (23, String::from("BP 24 dB")),
        (22, String::from("BP OB-Xd 12 dB")),
        (19, String::from("BP Cutoff Warp")),
        (31, String::from("BP Res Warp")),
        (4, String::from("HP 12 dB")),
        (5, String::from("HP 24 dB")),
        (14, String::from("HP K35")),
        (20, String::from("HP OB-Xd 12 dB")),
        (17, String::from("HP Cutoff Warp")),
        (29, String::from("HP Resonance Warp")),
        (7, String::from("Notch 12 dB")),
        (24, String::from("Notch 24 dB")),
        (21, String::from("Notch OB-Xd 12 dB")),
        (18, String::from("Notch Cutoff Warp")),
        (30, String::from("Notch Resonance Warp")),
        (33, String::from("Multi Tripole")),
        (36, String::from("FX Allpass")),
        (27, String::from("FX Cutoff Warp AP")),
        (32, String::from("FX Resonance Warp AP")),
        (8, String::from("FX Comb+")),
        (25, String::from("FX Comb-")),
        (9, String::from("FX Sample & Hold")),
    ]);
}

pub(crate) async fn extension_filter(
    peer_recv: &Peer,
    peer_send: &Peer,
    message: OscMessage,
) -> Result<OscMessage> {
    let result = match message {
        msg if ADDR_PATTERNS
            .get("filter_type")
            .unwrap()
            .is_match(message.addr.as_str()) =>
        {
            translate_filter_type(peer_recv, peer_send, msg).await
        }
        _ => message,
    };

    Ok(result)
}

/// Translates float to string for filter type
async fn translate_filter_type(
    peer_recv: &Peer,
    peer_send: &Peer,
    message: OscMessage,
) -> OscMessage {
    // Translation towards a controller
    match peer_send.kind {
        PeerKind::Controller => {
            let arg = message.args.first().expect("Can't find first argument!");
            let arg_value = match arg {
                async_osc::OscType::Int(res) => *res,
                async_osc::OscType::Float(res) => *res as i32,
                async_osc::OscType::String(s) => s.parse::<i32>().unwrap_or_default(),
                async_osc::OscType::Long(l) => *l as i32,
                async_osc::OscType::Double(d) => *d as i32,
                async_osc::OscType::Char(c) => *c as i32,
                // Any other type means we don't know what to do so just pass the message on
                _ => {
                    return message;
                }
            };
            if let Some(value) = FILTERTYPES.get(&arg_value) {
                debug!("Translated filter type for {}", peer_send.kind);
                return OscMessage::new(message.addr.clone(), vec![value.to_string()]);
            }
        }
        PeerKind::Instrument => {
            let arg = message.args.first().expect("Can't find first argument!");
            let arg_value = match arg {
                async_osc::OscType::String(s) => s,
                // Any other type means we don't know what to do so just pass the message on
                _ => {
                    return message;
                }
            };
            if let Some(value) = reverse_lookup(arg_value) {
                debug!("Translated filter type for {}", peer_send.kind);
                return OscMessage::new(message.addr.clone(), vec![value.to_string()]);
            }
        }
    }

    message
}

fn reverse_lookup(value: &String) -> Option<i32> {
    match FILTERTYPES.iter().find(|f| f.1 == value) {
        Some(kv_pair) => Some(*kv_pair.0),
        None => None,
    }
}
