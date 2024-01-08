use crate::{config::Config, labeler::LabeledMessage};
use log::{debug, warn};
use rosc::OscType;
use std::{
    io::{self, Error},
    path::PathBuf,
    sync::Arc,
};

use super::build_return_message;

/// Checks if the patchbay is occupied and returns bool to sender
pub(super) fn check_patchbay(
    config: Arc<Config>,
    labeled: LabeledMessage,
) -> Result<LabeledMessage, io::Error> {
    let requested_patchbay = get_patchbay(&labeled)?;
    let patchbay_path = guarantee_patch_path(config, &requested_patchbay)?;
    let return_bool = match retrieve_patch_filename_from_bay(&patchbay_path) {
        Ok(_) => true,
        Err(_) => false,
    };
    debug!(
        "Patchbay {} check in {:?} returned {}",
        requested_patchbay, patchbay_path, return_bool
    );
    let result = build_return_message(
        labeled,
        format!("/sys/patchbay/check/{}", requested_patchbay),
        OscType::Bool(return_bool),
    );

    Ok(result)
}

/// Loads a patch from the given patchbay.
/// Uses the first .fxp file it can find!
pub(super) fn load_patch(
    config: Arc<Config>,
    labeled: LabeledMessage,
) -> Result<LabeledMessage, io::Error> {
    let patchbay = get_patchbay(&labeled)?;
    debug!("Starting load process for patchbay {}", patchbay);
    let patch_path = guarantee_patch_path(config, &patchbay)?;

    let found_patch_name = retrieve_patch_filename_from_bay(&patch_path)?;

    // Now we message Surge to save the current patch to this path.
    let path_with_filename = format!(
        "{}{}",
        patch_path.to_string_lossy().to_string(),
        found_patch_name
    );
    let message = rosc::OscMessage {
        addr: String::from("/patch/load"),
        args: vec![OscType::String(path_with_filename)],
    };
    Ok(LabeledMessage {
        message,
        peer_recv: labeled.peer_recv,
        peer_send: labeled.peer_send,
    })
}

/// Saves a patch in the given patchbay
/// Removes all existing .fxp files from patchbay dir!
pub(super) fn save_patch(
    config: Arc<Config>,
    labeled: LabeledMessage,
) -> Result<LabeledMessage, io::Error> {
    // Create some basic information
    let patchbay = get_patchbay(&labeled)?;
    let current_patch_name = get_patchname(&labeled)?;
    let patch_path = guarantee_patch_path(config, &patchbay)?;

    // Clear out the patchbay
    clear_patchbay(&patch_path)?;

    // Now we message Surge to save the current patch to this path.
    let path_with_filename = format!(
        "{}{}",
        patch_path.to_string_lossy().to_string(),
        current_patch_name
    );
    debug!("Asking instrument to save patch to {}", path_with_filename);

    let message = rosc::OscMessage {
        addr: String::from("/patch/save"),
        args: vec![OscType::String(path_with_filename)],
    };
    Ok(LabeledMessage {
        message,
        peer_recv: labeled.peer_recv,
        peer_send: labeled.peer_send,
    })
}

// ********
// Helpers
// ********

fn guarantee_patch_path(config: Arc<Config>, patchbay: &String) -> io::Result<PathBuf> {
    let patch_path: PathBuf = {
        let mut path = dirs::config_local_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not find home directory")
        })?;
        let subdir = match &config.options.patch_cache_path.strip_prefix('/') {
            Some(dir) => dir.to_string(),
            None => config.options.patch_cache_path.to_owned(),
        };
        path.push(subdir);
        path.push(format!("{}/", patchbay));
        path
    };
    // If they don't exist, try to create dir and parent dirs
    // If we fail, error.
    if !patch_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&patch_path) {
            warn!("Failed to create directory {:?}: {}", patch_path, e);
            return Err(Error::new(
                io::ErrorKind::Other,
                format!("Failed to create directory: {}", e),
            ));
        }
        debug!("Created patchbay directory for {:?}", patch_path);
    }

    Ok(patch_path)
}

fn get_patchbay(labeled: &LabeledMessage) -> io::Result<String> {
    labeled
        .message
        .args
        .get(0)
        .and_then(|f| f.to_owned().string())
        .ok_or_else(|| {
            Error::new(
                io::ErrorKind::NotFound,
                "Not enough arguments given. Needs 2.",
            )
        })
}

fn get_patchname(labeled: &LabeledMessage) -> io::Result<String> {
    let pathstr = labeled
        .message
        .args
        .get(1)
        .and_then(|arg| arg.clone().string())
        .ok_or_else(|| {
            Error::new(
                io::ErrorKind::NotFound,
                "Second argument not found or not a string.",
            )
        })?;

    let result = PathBuf::from(pathstr)
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File name not found in the path."))?
        .to_owned()
        .to_string_lossy()
        .to_string();

    Ok(result)
}

fn clear_patchbay(path: &PathBuf) -> Result<(), io::Error> {
    // Delete all sfx files in this cache dir. When we load from this patchbay we take
    // the first and only patch we find in there.
    let old_patch_files = std::fs::read_dir(&path)?.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.extension()? == "fxp" {
            Some(path)
        } else {
            None
        }
    });
    for patch_file in old_patch_files {
        if let Err(e) = std::fs::remove_file(&patch_file) {
            warn!("Failed to delete file {:?}: {}", patch_file, e);
            return Err(Error::new(
                io::ErrorKind::Other,
                format!("Failed to delete file: {}", e),
            ));
        }
    }

    Ok(())
}

fn retrieve_patch_filename_from_bay(path: &PathBuf) -> Result<String, io::Error> {
    let filename = std::fs::read_dir(&path)?
        .filter_map(Result::ok)
        .find_map(|entry| {
            let path = entry.path();
            if path.extension()? == "fxp" {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "No .fxp file found in the patch path",
            )
        })?;

    Ok(filename)
}
