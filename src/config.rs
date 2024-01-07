use crate::peer::Peer;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Options {
    pub extend: bool,
    pub dryrun: bool,
    pub patch_cache_path: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub options: Options,
    pub controller: Peer,
    pub instrument: Peer,
}

pub(crate) fn read_config_from_file(path: &PathBuf) -> io::Result<Config> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    match toml::from_str::<Config>(contents.as_str()) {
        Ok(config) => Ok(config),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
    }
}
