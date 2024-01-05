use crate::osc;
use crate::{labeler::LabeledMessage, peer::PeerKind};
use lazy_static::lazy_static;
use log::debug;
use std::collections::HashMap;
use std::io;

lazy_static! {
    static ref FX_TYPES: HashMap<i32, String> = {
        let effects = vec![
        "Off",
        "Delay",
        "Reverb 1",
        "Phaser",
        "Rotary",
        "Distortion",
        "EQ",
        "Freq Shift",
        "Conditioner",
        "Chorus",
        "Vocoder",
        "Reverb 2",
        "Flanger",
        "Ring Mod",
        "Airwindows",
        "Neuron",
        "Graphic EQ",
        "Resonator",
        "CHOW",
        "Exciter",
        "Ensemble",
        "Combulator",
        "Nimbus",
        "Tape",
        "Treemonster",
        "Waveshaper",
        "Mid-Side Tool",
        "Spring Reverb",
        "Bonsai",
        "Audio In",
    ];

    // Create a HashMap
    let effects_map: HashMap<i32, String> = effects
        .iter()
        .enumerate()
        .map(|(index, &effect)| (index as i32, effect.to_string()))
        .collect();
    effects_map
    };
}

/// Translates float to string for filter type
pub(super) fn translate_fx_type(mut labeled: LabeledMessage) -> Result<LabeledMessage, io::Error> {
    match labeled.peer_send.kind {
        // Translation towards a controller
        PeerKind::Controller => {
            let arg = labeled
                .message
                .args
                .first()
                .expect("Can't find first argument!");
            // Get the key for the fx type
            let arg_value = match arg {
                osc::Type::Int(res) => *res as i32,
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

            // If we find a value for this key, alter the message.
            if let Some(value) = FX_TYPES.get(&arg_value) {
                debug!("Translated filter type for {}", labeled.peer_send.kind);
                if let Some(arg) = labeled.message.args.get_mut(0) {
                    *arg = osc::Type::String(value.to_string());
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
    match FX_TYPES.iter().find(|f| f.1 == value) {
        Some(kv_pair) => Some(*kv_pair.0),
        None => None,
    }
}
