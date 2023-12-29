use clap::{value_parser, Arg, Command};
use config::read_config_from_file;

use std::{path::PathBuf, sync::Arc};

use crate::handler::listening_tasks;

mod config;
mod extension;
mod handler;
mod peer;

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

    listening_tasks(config)
}
