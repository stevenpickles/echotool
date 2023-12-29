use clap::{command, Arg};

pub fn parse_command_line_args() -> clap::ArgMatches {
    command!()
        .arg(
            Arg::new("remote_url")
                .help("the remote IP address or URL to connect to (client mode only); omit for server modes")
                .display_order(1),
        )
        .arg(
            Arg::new("remote_port")
                .short('r')
                .long("remote_port")
                .value_parser(clap::value_parser!(u16).range(1..))
                .default_value("7")
                .help("the remote port to connect to (client modes only)"),
        )
        .arg(
            Arg::new("local_port")
                .short('l')
                .long("local_port")
                .value_parser(clap::value_parser!(u16).range(1..))
                .default_value("7")
                .help("the local port to bind to (client and server modes)"),
        )
        .arg(
            Arg::new("data_payload")
                .short('d')
                .long("data_payload")
                .default_value("Hello World!")
                .help("the data payload to send (client modes only)"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_parser(clap::value_parser!(u32))
                .default_value("5")
                .help("the number of times to send the data payload (client modes only)"),
        )
        .arg(
            Arg::new("timeout_in_seconds")
                .short('t')
                .long("timeout")
                .value_parser(clap::value_parser!(f64))
                .default_value("1.0")
                .help("the timeout in seconds (client modes only)"),
        )
        .arg(
            Arg::new("protocol")
                .short('p')
                .long("protocol")
                .value_parser(["udp", "tcp"])
                .default_value("udp")
                .help("the protocol to use (client and server modes)")
        )
        .get_matches()
}
