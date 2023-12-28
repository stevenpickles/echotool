use log::{error, info};
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::time::{sleep, timeout, Duration};

pub async fn server_task(local_port: u16) {
    info!("server start");

    let addr = format!("0.0.0.0:{local_port}");
    let result = UdpSocket::bind(&addr).await;
    let socket = match result {
        Ok(socket) => socket,
        Err(e) => {
            error!("failed to bind socket: {e}");
            return;
        }
    };
    info!("server listening on {addr}");

    let mut count = 0;

    tokio::select! {
        () = async {
            loop {
                let mut buf: [u8; 65536] = [0u8; 65536];
                match socket.recv_from(&mut buf).await {
                    Ok((size, src)) => {
                        count += 1;
                        info!("[{count}] received {size} bytes from {src}");

                        match socket.send_to(&buf[0..size], &src).await {
                            Ok(size) => {
                                info!("sent {size} bytes to {src}");
                            }
                            Err(e) => {
                                error!("error sending data: {e}");
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("error while receiving data: {e}");
                        return;
                    }
                }
            }
        } => {}

        _ = tokio::signal::ctrl_c() => {
            // Handle Ctrl+C event
            info!("Received Ctrl+C. Aborting the server task.");
        }
    }

    info!("server stop");
}

pub async fn client_task(
    remote_url: String,
    remote_port: u16,
    local_port: u16,
    count: u32,
    timeout_in_seconds: f64,
    data_payload: String,
) {
    info!("udp client start");

    // Specify the local and remote addresses
    let local_addr = format!("0.0.0.0:{local_port}");
    let remote_addr = format!("{remote_url}:{remote_port}");

    // Specify the payload for the UDP packet
    let payload = data_payload.as_bytes();

    // Create a signal stream for Ctrl+C
    let ctrl_c = signal::ctrl_c();

    // Call the function to send and receive the UDP echo packet
    let continue_forever = count == 0;
    let mut remaining = count;

    tokio::select! {
        () = async {
            while (remaining > 0) || continue_forever {
                if let Err(e) = send_receive_echo_packet(
                    local_addr.clone(),
                    remote_addr.clone(),
                    payload,
                    timeout_in_seconds,
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

    info!("udp client stop");
}

pub async fn send_receive_echo_packet(
    local_addr: String,
    remote_addr: String,
    payload: &[u8],
    timeout_in_seconds: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a UDP socket bound to the specified local address
    let socket = UdpSocket::bind(local_addr).await?;

    // Send the UDP packet with the specified payload to the remote address
    info!("sending {} bytes to {remote_addr}", payload.len());
    let result = timeout(
        Duration::from_secs_f64(timeout_in_seconds),
        socket.send_to(payload, &remote_addr),
    )
    .await;
    match result {
        Ok(Ok(size)) => {
            info!("sent {size} bytes to {remote_addr}");
        }
        Ok(Err(e)) => {
            return Err(format!("error sending data: {e}").into());
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
        socket.recv_from(&mut buffer),
    )
    .await;

    match result {
        Ok(Ok((num_bytes, _))) => {
            info!("received {num_bytes} bytes");
            let received_payload = &buffer[..num_bytes];
            if received_payload == payload {
                info!("payloads match");
            } else {
                info!("payloads do not match");
            }
        }
        Ok(Err(e)) => {
            error!("error receiving data: {e}");
        }
        Err(_) => {
            error!("timeout: no response received within {timeout_in_seconds} seconds");
        }
    }

    Ok(())
}
