use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tokio::time::{sleep, timeout, Duration};

pub async fn server_task(local_port: u16) {
    info!("server start");

    let addr = format!("0.0.0.0:{local_port}");
    let result = TcpListener::bind(&addr).await;
    let listener = match result {
        Ok(listener) => listener,
        Err(e) => {
            error!("failed to bind listener: {e}");
            return;
        }
    };
    info!("server listening on {addr}");

    let mut count = 0;

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
}

pub async fn client_task(
    remote_url: String,
    remote_port: u16,
    local_port: u16,
    count: u32,
    timeout_in_seconds: f64,
    data_payload: String,
) {
    info!("tcp client start");

    // Specify the local and remote addresses
    let _local_addr = format!("0.0.0.0:{local_port}");
    let remote_addr = format!("{remote_url}:{remote_port}");

    // Specify the payload for the packet
    let payload = data_payload.as_bytes();

    let result = timeout(
        Duration::from_secs_f64(timeout_in_seconds),
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
            error!("timeout: no response received within {timeout_in_seconds} seconds");
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

    let continue_forever = count == 0;

    let mut remaining = count;
    while (remaining > 0) || continue_forever {
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
                error!("error sending data: {e}");
                return;
            }
            Err(_) => {
                error!("timeout: no response received within {timeout_in_seconds} seconds");
                return;
            }
        }

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
                error!("failed to read from socket: {e}");
                return;
            }
            Err(_) => {
                error!("timeout: no response received within {timeout_in_seconds} seconds");
                return;
            }
        }

        remaining = remaining.saturating_sub(1);

        sleep(Duration::from_millis(100)).await;
    }

    info!("tcp client stop");
}
