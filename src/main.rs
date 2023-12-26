use clap::{command, Arg};

fn main() {
    let match_result = command!()
    .arg(
        Arg::new("remote_url")
            .help("the remote URL to connect to (client mode only)")
    )
    .arg(
        Arg::new("remote_port")
            .short('r')
            .long("remote-port")
    )
    .arg(
        Arg::new("local_port")
            .short('l')
            .long("local-port")
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
}