use crate::{config::Config, labeler::LabeledMessage, osc};
use log::{debug, warn};
use rosc::OscType;
use std::{io, sync::Arc};
mod patchbay;

/// System messages are addressed to Arcflash i.e. the packet router.
/// If the packet router runs on the system the instrument runs on, then system
/// diagnostics from the router can be used to indicate the instruments health too.
/// System messages are always returned to the the peer they were received from.
pub fn system_handler(
    config: Arc<Config>,
    labeled: LabeledMessage,
) -> Result<LabeledMessage, io::Error> {
    // System average load
    if labeled.message.addr.contains("/sys/q/system_load") {
        let addr = String::from("/sys/system_load");

        if let Ok(load) = sys_info::loadavg() {
            let load_message = OscType::String(format!(
                "1 min: {:.2}, 5 min: {:.2}, 15 min: {:.2}",
                load.one, load.five, load.fifteen
            ));
            let return_message = build_return_message(labeled, addr, load_message);
            return Ok(return_message);
        }
    }

    // Cpu speed
    if labeled.message.addr.contains("/sys/q/cpu_speed") {
        let addr = String::from("/sys/cpu_speed");

        match sys_info::cpu_speed() {
            Ok(cpu_speed) => {
                let load_message = OscType::String(String::from(format!("{} mhz", cpu_speed)));
                let return_message = build_return_message(labeled, addr, load_message);
                return Ok(return_message);
            }
            Err(e) => {
                warn!("Unable to get cpu speed.");
                let error_msg = OscType::String(String::from(format!("Error: {}", e)));
                return Ok(build_return_message(labeled, addr, error_msg));
            }
        }
    }

    // Is arcflash enabled?
    if labeled.message.addr.contains("/sys/q/arcflash") {
        let addr = String::from("/sys/arcflash");

        let load_message = OscType::Bool(true);
        let return_message = build_return_message(labeled, addr, load_message);
        return Ok(return_message);
    }

    // Handle loading and saving to patch bays
    if labeled.message.addr.contains("/sys/patchbay/save") {
        return patchbay::save_patch(config, labeled);
    };
    if labeled.message.addr.contains("/sys/patchbay/load") {
        return patchbay::load_patch(config, labeled);
    };
    if labeled.message.addr.contains("/sys/patchbay/check") {
        return patchbay::check_patchbay(config, labeled);
    };

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

fn build_return_message(labeled: LabeledMessage, addr: String, content: OscType) -> LabeledMessage {
    let mut return_message = labeled.clone();
    return_message.peer_send = return_message.peer_recv.clone();
    return_message.message = osc::Message {
        addr,
        args: vec![content],
    };
    return_message
}
