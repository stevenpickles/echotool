use clap::{command, Arg};
use log::{info, error, LevelFilter};
use env_logger::Builder;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::time::{sleep, Duration};


#[tokio::main]
async fn main() {
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

    if is_client_mode {
        let client_task: tokio::task::JoinHandle<()> = tokio::spawn( client_thread() );
        signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
        client_task.abort();
    }
    else {
        let server_task: tokio::task::JoinHandle<()> = tokio::spawn( server_thread( *local_port) );
        signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
        server_task.abort();
    }



    sleep(Duration::from_millis(2500)).await;

    info!("application end");
    
}

async fn server_thread(local_port: u16) {
    info!("server start");

    let addr = format!( "0.0.0.0:{}", local_port );
    let socket = UdpSocket::bind(&addr).await.expect( "failed to bind socket" );
    info!("server listening on {}", addr);

    let mut buf: [u8; 65536] = [0u8; 65536];
    let mut count = 0;

    

    loop {
        tokio::select! {
            recv_result = socket.recv_from(&mut buf) => match recv_result {
                Ok((size, src)) => {
                    count += 1;
                    info!("[{}] received {} bytes from {}", count, size, src);
                }
                Err(e) => {
                    error!("error while receiving data: {}", e);
                }
            },
            _ = tokio::signal::ctrl_c() => {
                info!("detected ctrl+c, shutting down...");
                break;
            }
        }
    }

    info!("server stop")
}

async fn client_thread() {
    info!("client start");

    info!("client stop")
}
