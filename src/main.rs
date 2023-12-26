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
    .arg(
        Arg::new("data_payload")
            .short('d')
            .long("data_payload")
            .default_value("Hello World!")
    )
    .arg(
        Arg::new("count")
            .short('c')
            .long("count")
            .value_parser(clap::value_parser!(u32))
            .default_value("5")
    )
    .get_matches();

    let mut remote_url = "";
    let is_client_mode = match_result.contains_id("remote_url");
    if is_client_mode {
        remote_url = match_result.get_one::<String>("remote_url").unwrap();
        println!("remote_url is {} -- client mode enabled", remote_url);
    }
    else {
        println!("remote_url is empty -- server mode enabled");
    
    }

    let remote_port = match_result.get_one::<u16>("remote_port").unwrap();
    println!("remote_port is {}", remote_port);

    let local_port = match_result.get_one::<u16>("local_port").unwrap();
    println!("local_port is {}", local_port);

    let data_payload = match_result.get_one::<String>("data_payload").unwrap();

    let count = match_result.get_one::<u32>("count").unwrap();

    // Initialize the logger (env_logger)
    let mut builder = Builder::new();
    builder.format_timestamp_micros();
    builder.filter_level(LevelFilter::Info);
    builder.target(env_logger::Target::Stdout);
    builder.init();

    info!("application start");

    if is_client_mode {
        client_task( remote_url.to_string(), *remote_port, *local_port, *count, data_payload.to_string() ).await;
    }
    else {
        let server_task: tokio::task::JoinHandle<()> = tokio::spawn( server_thread( *local_port ) );
        signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
        server_task.abort();
        sleep(Duration::from_millis(1000)).await;
    }

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
                    socket.send_to(&buf[0..size], &src).await.expect("failed to send data");
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

async fn client_task(remote_url: String, remote_port: u16, local_port: u16, count: u32, data_payload: String) {
    info!("client start");

    // Specify the local and remote addresses
    let local_addr = format!( "0.0.0.0:{}", local_port );
    let remote_addr = format!( "{}:{}", remote_url, remote_port );

    // Specify the payload for the UDP packet
    let payload = data_payload.as_bytes();

    // Call the function to send and receive the UDP echo packet
    for _i in 0..count {
        if let Err(e) = send_receive_udp_echo_packet(local_addr.clone(), remote_addr.clone(), payload).await {
            error!("Error: {}", e);
        }
        sleep(Duration::from_millis(1000)).await;
    }

    info!("client stop")
}


async fn send_receive_udp_echo_packet(
    local_addr: String,
    remote_addr: String,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a UDP socket bound to the specified local address
    let socket = UdpSocket::bind(local_addr).await?;

    // Send the UDP packet with the specified payload to the remote address
    info!("sending {} bytes to {}", payload.len(), remote_addr);
    socket.send_to(payload, &remote_addr).await?;

    // Buffer to store the received data
    let mut buf = vec![0; payload.len()];

    // Receive the response from the server
    info!("receiving response...");
    let (num_bytes, _) = socket.recv_from(&mut buf).await?;
    info!("received {} bytes", num_bytes);

    // Compare the received payload with the transmitted payload
    let received_payload = &buf[..num_bytes];
    if received_payload == payload {
        info!("payloads match");
    } else {
        info!("payloads do not match");
    }

    Ok(())
}