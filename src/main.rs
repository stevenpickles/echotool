use clap::{command, Arg};
use log::{info, LevelFilter};
use env_logger::Builder;

fn main() {
    let match_result = command!()
    .arg(
        Arg::new("remote_url")
            .help("the remote URL to connect to (client mode only)")
            .display_order(1)
    )
    .arg(
        Arg::new("remote_port")
            .short('r')
            .long("remote_port")
            .value_parser(clap::value_parser!(u16).range(1..))
            .default_value("7")
    )
    .arg(
        Arg::new("local_port")
            .short('l')
            .long("local_port")
            .value_parser(clap::value_parser!(u16).range(1..))
            .default_value("7")
    )
    .get_matches();

    let is_client_mode = match_result.contains_id("remote_url");
    if is_client_mode {
        let remote_url = match_result.get_one::<String>("remote_url").unwrap();
        println!("remote_url is {} -- client mode enabled", remote_url);
    }
    else {
        println!("remote_url is empty -- server mode enabled");
    
    }

    let remote_port = match_result.get_one::<u16>("remote_port").unwrap();
    println!("remote_port is {}", remote_port);

    let local_port = match_result.get_one::<u16>("local_port").unwrap();
    println!("local_port is {}", local_port);

    // Initialize the logger (env_logger)
    let mut builder = Builder::new();
    builder.format_timestamp_micros();
    builder.filter_level(LevelFilter::Info);
    builder.target(env_logger::Target::Stdout);
    builder.init();

    info!("application start");
}