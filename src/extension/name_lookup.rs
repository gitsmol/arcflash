use crate::osc;
use crate::{labeler::LabeledMessage, peer::PeerKind};
use log::debug;

use std::collections::HashMap;
use std::io;

/// Translates float to string for filter type
pub(super) fn lookup(
    mut labeled: LabeledMessage,
    map: &HashMap<i32, String>,
) -> Result<LabeledMessage, io::Error> {
    match labeled.peer_send.kind {
        // Translation towards a controller
        PeerKind::Controller => {
            let arg = labeled
                .message
                .args
                .first()
                .expect("Can't find first argument!");
            let arg_value = match arg {
                osc::Type::Int(res) => *res,
                osc::Type::Float(res) => *res as i32,
                osc::Type::String(s) => s.parse::<i32>().unwrap_or_default(),
                osc::Type::Long(l) => *l as i32,
                osc::Type::Double(d) => *d as i32,
                osc::Type::Char(c) => *c as i32,
                // Any other type means we don't know what to do so just pass the message on
                _ => {
                    return Ok(labeled);
                }
            };
            if let Some(value) = map.get(&arg_value) {
                debug!("Translated filter type for {}", labeled.peer_send.kind);
                if let Some(arg) = labeled.message.args.get_mut(0) {
                    *arg = osc::Type::String(value.to_owned());
                };
                return Ok(labeled);
            }
        }
        // Translation towards an instrument
        PeerKind::Instrument => {
            let arg = labeled
                .message
                .args
                .first()
                .expect("Can't find first argument!");
            let arg_value = match arg {
                osc::Type::String(s) => s,
                // Any other type means we don't know what to do so just pass the message on
                _ => {
                    return Ok(labeled);
                }
            };
            if let Some(value) = reverse_lookup(arg_value, map) {
                debug!("Translated filter type for {}", labeled.peer_send.kind);
                if let Some(arg) = labeled.message.args.get_mut(0) {
                    *arg = osc::Type::Int(value);
                };
                return Ok(labeled);
            }
        }
    }

    Ok(labeled)
}

fn reverse_lookup(value: &String, map: &HashMap<i32, String>) -> Option<i32> {
    match map.iter().find(|f| f.1 == value) {
        Some(kv_pair) => Some(*kv_pair.0),
        None => None,
    }
}
