use crate::osc;
use crate::{labeler::LabeledMessage, peer::PeerKind};
use lazy_static::lazy_static;
use log::debug;

use std::collections::HashMap;
use std::io;

lazy_static! {
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

/// Translates float to string for filter type
pub(super) fn translate_filter_type(
    mut labeled: LabeledMessage,
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
            if let Some(value) = FILTERTYPES.get(&arg_value) {
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
            if let Some(value) = reverse_lookup(arg_value) {
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

fn reverse_lookup(value: &String) -> Option<i32> {
    match FILTERTYPES.iter().find(|f| f.1 == value) {
        Some(kv_pair) => Some(*kv_pair.0),
        None => None,
    }
}
