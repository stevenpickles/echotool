use clap::{command, Arg};
use log::{info, error, LevelFilter};
use env_logger::Builder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::signal;
use tokio::time::{sleep, timeout, Duration};


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
    .arg(
        Arg::new("timeout_in_seconds")
            .short('t')
            .long("timeout")
            .value_parser(clap::value_parser!(f32))
            .default_value("1.0")
    )
    .arg(
        Arg::new("protocol")
            .short('p')
            .long("protocol")
            .value_parser( ["udp", "tcp"] )
            .default_value("udp")
    )
    .get_matches();

    // Initialize the logger (env_logger)
    let mut builder = Builder::new();
    builder.format_timestamp_micros();
    builder.filter_level(LevelFilter::Info);
    builder.target(env_logger::Target::Stdout);
    builder.init();

    let mut remote_url = "";
    let is_client_mode = match_result.contains_id("remote_url");
    if is_client_mode {
        remote_url = match_result.get_one::<String>("remote_url").unwrap();
        info!("remote_url is {} -- client mode enabled", remote_url);
    }
    else {
        info!("remote_url is empty -- server mode enabled");
    
    }

    let remote_port = match_result.get_one::<u16>("remote_port").unwrap();
    info!("remote_port is {}", remote_port);

    let local_port = match_result.get_one::<u16>("local_port").unwrap();
    info!("local_port is {}", local_port);

    let data_payload = match_result.get_one::<String>("data_payload").unwrap();
    info!("data_payload is {}", data_payload);

    let count = match_result.get_one::<u32>("count").unwrap();
    info!("count is {}", count);

    let protocol = match_result.get_one::<String>("protocol").unwrap();
    info!("protocol is {}", protocol);

    let mut timeout_in_seconds = match_result.get_one::<f32>("timeout_in_seconds").unwrap();
    if timeout_in_seconds < &0.2 {
        timeout_in_seconds = &0.2;
    }
    info!("timeout_in_seconds is {}", timeout_in_seconds);

    info!("application start");

    if is_client_mode {
        if protocol == "tcp" {
            client_task_tcp( remote_url.to_string(), *remote_port, *local_port, *count, *timeout_in_seconds, data_payload.to_string() ).await;
        }
        else {
            client_task_udp( remote_url.to_string(), *remote_port, *local_port, *count, *timeout_in_seconds, data_payload.to_string() ).await;
        }
    }
    else {
        if protocol == "tcp" {
            let server_task: tokio::task::JoinHandle<()> = tokio::spawn( server_thread_tcp( *local_port ) );
            signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
            server_task.abort();
        }
        else {
            let server_task: tokio::task::JoinHandle<()> = tokio::spawn( server_thread_udp( *local_port ) );
            signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
            server_task.abort();
        }
        sleep(Duration::from_millis(1000)).await;
    }

    info!("application end");
    
}

async fn server_thread_tcp(local_port: u16) {
    info!("server start");

    let addr = format!( "0.0.0.0:{}", local_port );
    let listener = TcpListener::bind(&addr).await.expect( "failed to create listener" );
    info!("server listening on {}", addr);

    let mut count = 0;

    loop {
        let ( mut socket, _) = listener.accept().await.expect( "failed to accept connection");
        info!("server connected to {}", socket.peer_addr().unwrap() );

        tokio::spawn( async move {

            loop {
                let mut buf: [u8; 65536] = [0u8; 65536];
                match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        info!("connection closed by client {}", socket.peer_addr().unwrap());
                        break;
                    }
                    Ok(n) => {
                        count += 1;
                        info!("[{}] received {} bytes from {}", count, n, socket.peer_addr().unwrap() );

                        if let Err(e) = socket.write_all(&buf[..n]).await {
                            error!("error writing to socket: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("error while receiving data: {}", e);
                    }
                }
            }
        } );
    }
}

async fn server_thread_udp(local_port: u16) {
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

async fn client_task_udp(remote_url: String, remote_port: u16, local_port: u16, count: u32, timeout_in_seconds: f32, data_payload: String) {
    info!("client start");

    // Specify the local and remote addresses
    let local_addr = format!( "0.0.0.0:{}", local_port );
    let remote_addr = format!( "{}:{}", remote_url, remote_port );

    // Specify the payload for the UDP packet
    let payload = data_payload.as_bytes();

    // Call the function to send and receive the UDP echo packet
    if count == 0 {
        loop {
            if let Err(e) = send_receive_udp_echo_packet(local_addr.clone(), remote_addr.clone(), payload, timeout_in_seconds).await {
                error!("error: {}", e);
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
    else {
        for _i in 0..count {
            if let Err(e) = send_receive_udp_echo_packet(local_addr.clone(), remote_addr.clone(), payload, timeout_in_seconds).await {
                error!("error: {}", e);
            }
            sleep(Duration::from_millis(100)).await;
        }
    }


    info!("client stop")
}


async fn send_receive_udp_echo_packet(
    local_addr: String,
    remote_addr: String,
    payload: &[u8],
    timeout_in_seconds: f32,
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
    let result = timeout(Duration::from_secs_f32(timeout_in_seconds), socket.recv_from(&mut buf)).await;

    match result {
        Ok(Ok((num_bytes, _))) => {
            info!("received {} bytes", num_bytes);
            let received_payload = &buf[..num_bytes];
            if received_payload == payload {
                info!("payloads match");
            } else {
                info!("payloads do not match");
            }
        }
        Ok(Err(e)) => {
            error!("error receiving data: {}", e);
        }
        Err(_) => {
            error!("timeout: no response received within {} seconds", timeout_in_seconds);
        }
    }

    Ok(())
}


async fn client_task_tcp(
    remote_url: String,
    remote_port: u16,
    local_port: u16,
    count: u32,
    timeout_in_seconds: f32,
    data_payload: String ) {
    info!("tcp client start");

    // Specify the local and remote addresses
    let local_addr = format!( "0.0.0.0:{}", local_port );
    let remote_addr = format!( "{}:{}", remote_url, remote_port );

    // Specify the payload for the packet
    let payload = data_payload.as_bytes();

    let result = timeout(Duration::from_secs_f32(timeout_in_seconds), TcpStream::connect(remote_addr) ).await;
    let mut stream = match result {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            error!("failed to connect: {}", e);
            return;
        }
        Err(_) => {
            error!("timeout: no response received within {} seconds", timeout_in_seconds);
            return;
        }
    };

    let mut continue_forever = false;
    if count == 0 {
        continue_forever = true;
    }

    let mut remaining = count;
    while ( remaining > 0 ) || continue_forever {
        info!("sending {} bytes to {}", payload.len(), stream.peer_addr().unwrap() );
        let result = stream.write_all(payload).await;
        match result {
            Ok(_) => {
                info!("sent {} bytes to {}", payload.len(), stream.peer_addr().unwrap() );
            }
            Err(e) => {
                error!("error sending data: {}", e);
                return;
            }
        }

        let mut buffer = vec![0; payload.len()];
        info!("receiving response...");
        let result = stream.read_exact(&mut buffer).await;
        match result {
            Ok(_) => {
                info!("received {} bytes", buffer.len());

                if buffer == payload {
                    info!("payloads match");
                }
                else {
                    info!("payloads do not match");
                }
            }
            Err(e) => {
                error!("failed to read from socket: {}", e);
                return;
            }
        }

        if remaining > 0 {
            remaining -= 1;
        }

        sleep(Duration::from_millis(100)).await;
    }

    info!("tcp client stop")
}