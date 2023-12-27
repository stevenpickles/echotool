use env_logger::Builder;
use log::{error, info, LevelFilter};
use tokio::signal;

mod clargs;
mod tcp;
mod udp;

#[tokio::main]
async fn main() {
    init_logger();

    // Parse command-line arguments
    let clargs = clargs::parse_command_line_args();

    let mut remote_url = "";
    let is_client_mode = clargs.contains_id("remote_url");
    if is_client_mode {
        let result = clargs.get_one::<String>("remote_url");
        result.map_or_else(
            || {
                info!("remote_url is empty -- server mode enabled");
            },
            |value| {
                remote_url = value;
                info!("remote_url is {remote_url} -- client mode enabled");
            },
        );
    } else {
        info!("remote_url is empty -- server mode enabled");
    }

    let Some(remote_port) = clargs.get_one::<u16>("remote_port") else {
        error!("remote_port is invalid");
        return;
    };
    info!("remote_port is {remote_port}");

    let Some(local_port) = clargs.get_one::<u16>("local_port") else {
        error!("local_port is invalid");
        return;
    };
    info!("local_port is {local_port}");

    let Some(data_payload) = clargs.get_one::<String>("data_payload") else {
        error!("data_payload is invalid");
        return;
    };
    info!("data_payload is {data_payload}");

    let Some(count) = clargs.get_one::<u32>("count") else {
        error!("count is invalid");
        return;
    };
    info!("count is {count}");

    let Some(protocol) = clargs.get_one::<String>("protocol") else {
        error!("protocol is invalid");
        return;
    };
    info!("protocol is {protocol}");

    let Some(timeout_in_seconds) = clargs.get_one::<f64>("timeout_in_seconds") else {
        error!("timeout_in_seconds is invalid");
        return;
    };
    info!("timeout_in_seconds is {timeout_in_seconds}");

    info!("application start");

    // Delegate to appropriate functions based on protocol and mode
    if is_client_mode {
        if protocol == "tcp" {
            tcp::client_task(
                remote_url.to_string(),
                *remote_port,
                *local_port,
                *count,
                *timeout_in_seconds,
                data_payload.to_string(),
            )
            .await;
        } else {
            udp::client_task(
                remote_url.to_string(),
                *remote_port,
                *local_port,
                *count,
                *timeout_in_seconds,
                data_payload.to_string(),
            )
            .await;
        }
    } else if protocol == "tcp" {
        let server_task = tokio::spawn(tcp::server_task(*local_port));
        let result = signal::ctrl_c().await;
        match result {
            Ok(()) => {
                info!("detected ctrl+c, shutting down...");
            }
            Err(e) => {
                error!("failed to install ctrl+c signal handler: {e}");
                return;
            }
        }
        server_task.abort();
    } else {
        let server_task = tokio::spawn(udp::server_task(*local_port));
        let result = signal::ctrl_c().await;
        match result {
            Ok(()) => {
                info!("detected ctrl+c, shutting down...");
            }
            Err(e) => {
                error!("failed to install ctrl+c signal handler: {e}");
                return;
            }
        }
        server_task.abort();
    }

    info!("application end");
}


fn init_logger() {
    // Initialize the logger (env_logger)
    let mut builder = Builder::new();
    builder.format_timestamp_micros();
    builder.filter_level(LevelFilter::Info);
    builder.target(env_logger::Target::Stdout);
    builder.init();
}