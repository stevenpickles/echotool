use crate::app_config::AppConfig;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::time::{sleep, timeout, Duration};

pub async fn server_task(config: &AppConfig) {
    info!("tcp server start");

    let addr = format!("0.0.0.0:{}", config.local_port);
    let result = TcpListener::bind(&addr).await;
    let listener = match result {
        Ok(listener) => listener,
        Err(e) => {
            error!("failed to bind listener: {e}");
            return;
        }
    };
    info!("tcp server listening on {addr}");

    let mut count = 0;

    tokio::select! {
        () = async {
            loop {
                let result = listener.accept().await;
                let (mut socket, peer_addr) = match result {
                    Ok((socket, peer_addr)) => (socket, peer_addr),
                    Err(e) => {
                        error!("failed to accept connection: {e}");
                        return;
                    }
                };
                info!("server connected to {peer_addr}");

                tokio::spawn(async move {
                    loop {
                        let mut buf: [u8; 65536] = [0u8; 65536];
                        match socket.read(&mut buf).await {
                            Ok(0) => {
                                info!("connection closed by client {peer_addr}");
                                break;
                            }
                            Ok(n) => {
                                count += 1;
                                info!("[{count}] received {n} bytes from {peer_addr}");

                                if let Err(e) = socket.write_all(&buf[..n]).await {
                                    error!("error writing to socket: {e}");
                                    return;
                                }
                            }
                            Err(e) => {
                                error!("error while receiving data: {e}");
                                return;
                            }
                        }
                    }
                });
            }
        } => {}

        _ = signal::ctrl_c() => {
            // Handle Ctrl+C event
            info!("detected ctrl+c, shutting down...");
        }
    }

    info!("tcp server stop");
}

pub async fn client_task(config: &AppConfig) {
    info!("tcp client start");

    // Specify the local and remote addresses
    let _local_addr = format!("0.0.0.0:{}", config.local_port);
    let remote_addr = format!("{}:{}", config.remote_url, config.remote_port);

    // Specify the payload for the packet
    let payload = config.data_payload.as_bytes();

    // Create a signal stream for Ctrl+C
    let ctrl_c = signal::ctrl_c();

    // Establish TCP connection
    let result = timeout(
        Duration::from_secs_f64(config.timeout_in_seconds),
        TcpStream::connect(remote_addr),
    )
    .await;
    let mut stream = match result {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            error!("failed to connect: {e}");
            return;
        }
        Err(_) => {
            error!(
                "timeout: no response received within {} seconds",
                config.timeout_in_seconds
            );
            return;
        }
    };

    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!("failed to get peer address: {e}");
            return;
        }
    };
    info!("connected to peer {peer_addr}");

    // Call the function to send and receive data over the TCP connection
    let continue_forever = config.count == 0;
    let mut remaining = config.count;

    tokio::select! {
        () = async {
            while (remaining > 0) || continue_forever {
                if let Err(e) = send_receive_echo_packet(
                    &mut stream,
                    peer_addr.to_string(),
                    payload,
                    config.timeout_in_seconds,
                )
                .await
                {
                    error!("error: {e}");
                }

                remaining = remaining.saturating_sub(1);

                sleep(Duration::from_millis(100)).await;
            }
        } => {}

        _ = ctrl_c => {
            // Handle Ctrl+C event
            info!("detected ctrl+c, shutting down...");
        }
    }

    info!("tcp client stop");
}

pub async fn send_receive_echo_packet(
    stream: &mut TcpStream,
    peer_addr: String,
    payload: &[u8],
    timeout_in_seconds: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("sending {} bytes to {peer_addr}", payload.len());
    let result = timeout(
        Duration::from_secs_f64(timeout_in_seconds),
        stream.write_all(payload),
    )
    .await;
    match result {
        Ok(Ok(())) => {
            info!("sent {} bytes to {peer_addr}", payload.len());
        }
        Ok(Err(e)) => {
            return Err(format!("error sending data to socket: {e}").into());
        }
        Err(_) => {
            return Err(format!(
                "timeout: no response received within {timeout_in_seconds} seconds"
            )
            .into());
        }
    }

    // Receiving response
    let mut buffer = vec![0; payload.len()];
    info!("waiting for response...");
    let result = timeout(
        Duration::from_secs_f64(timeout_in_seconds),
        stream.read_exact(&mut buffer),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            info!("received {} bytes", buffer.len());
            if buffer == payload {
                info!("payloads match");
            } else {
                info!("payloads do not match");
            }
        }
        Ok(Err(e)) => {
            error!("error reading data: {e}");
        }
        Err(_) => {
            error!("timeout: no response received within {timeout_in_seconds} seconds");
        }
    }

    Ok(())
}
