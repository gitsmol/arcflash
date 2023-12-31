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
        .arg(
            Arg::new("log_to_file")
                .short('l')
                .long("log-file")
                .value_name("arcflash.log")
                // .default_value("./arcflash.log")
                .value_parser(value_parser!(PathBuf))
                .help("Log archflash output to file."),
        )
        .get_matches();

    // use std::env;

    if let Some(log_file) = matches.get_one::<PathBuf>("log_to_file") {
        let log_path = PathBuf::from(log_file);
        let file = std::fs::File::create(log_path).unwrap();
        let env = env_logger::Env::default();
        env_logger::Builder::from_env(env)
            .target(env_logger::Target::Pipe(Box::new(file)))
            .init();
    } else {
        env_logger::init();
    }

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
