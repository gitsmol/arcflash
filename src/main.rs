use crate::{config::read_config_from_file, handler::spawn_packet_handlers};
use clap::{value_parser, Arg, Command};
use std::{path::PathBuf, sync::Arc};

mod config;
mod extension;
mod handler;
mod labeler;
mod peer;
mod sender;

fn main() {
    let matches = Command::new("Arcflash")
        .version("0.1")
        .author("Geert Smolders <geert@polyprax.nl>")
        .about("Manages OSC connections between TouchOSC and Surge")
        .arg(
            Arg::new("extend")
                .short('e')
                .long("extended-features")
                .value_name("true")
                .value_parser(value_parser!(bool))
                .help("Extends the Surge OSC spec to provide additional features."),
        )
        .arg(
            Arg::new("config_file")
                .short('c')
                .long("config-file")
                .value_name("config.toml")
                .default_value("./config.toml")
                .value_parser(value_parser!(PathBuf))
                .help("Extends the Surge OSC spec to provide additional features."),
        )
        .get_matches();

    env_logger::init();

    let Some(config_file_path) = matches.get_one::<PathBuf>("config_file") else {
        panic!("Can't parse config file argument!")
    };
    let mut config = match read_config_from_file(config_file_path) {
        Ok(config) => config,
        Err(e) => panic!("Error reading config file: {}", e),
    };

    if let Some(value) = matches.get_one::<bool>("extend") {
        config.options.extend = *value;
    }

    println!("{:?}", config);
    let config = Arc::new(config);

    spawn_packet_handlers(config)
}
