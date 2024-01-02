use crate::{config::read_config_from_file, handler::spawn_handler, peer::PeerKind};
use clap::{value_parser, Arg, ArgMatches, Command};
use config::Config;
use log::info;
use std::{path::PathBuf, sync::Arc};

mod config;
mod extension;
mod handler;
mod labeler;
mod osc;
mod peer;
mod sender;
mod tests;

fn main() {
    let matches = read_command_line_args();
    init_logger(&matches);
    let config = create_config_arc(&matches);

    // Will proceed with tests and not run main program.
    run_tests(&matches);

    info!("Spawning handler threads.");

    // Threads for the packets coming from peers
    let t1 = spawn_handler(config.clone(), PeerKind::Instrument);
    let t2 = spawn_handler(config.clone(), PeerKind::Controller);

    while !t1.is_finished() && !t2.is_finished() {}

    info!("Shutting down.");
}

/// Startup logging and handle output to file
fn init_logger(matches: &ArgMatches) {
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
}

/// Create a config struct from file
fn create_config_arc(matches: &ArgMatches) -> Arc<Config> {
    let Some(config_file_path) = matches.get_one::<PathBuf>("config_file") else {
        panic!("Can't find config file!")
    };
    let mut config = match read_config_from_file(config_file_path) {
        Ok(config) => config,
        Err(e) => panic!("Error reading config file: {}", e),
    };

    if let Some(value) = matches.get_one::<bool>("extend") {
        config.options.extend = *value;
    }

    info!("Launching with config {:?}", config);
    Arc::new(config)
}

fn run_tests(matches: &ArgMatches) {
    if let Some(value) = matches.get_one::<bool>("test") {
        if value == &true {
            info!("Running tests.");
            panic!("Not implemented.")
            // tests::test_q_all_params();
        }
    }
}

/// Read command line args into matches
fn read_command_line_args() -> ArgMatches {
    Command::new("Arcflash")
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
        .arg(
            Arg::new("test")
                .short('t')
                .long("tests")
                .value_name("false")
                .default_value("false")
                .value_parser(value_parser!(bool))
                .help("Perform tests on OSC peers and arcflash throughput."),
        )
        .get_matches()
}
