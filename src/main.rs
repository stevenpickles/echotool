use env_logger::Builder;
use log::{error, info, LevelFilter};
use tokio::signal;

mod app_config;
mod clargs;
mod tcp;
mod udp;

#[tokio::main]
async fn main() {
    init_logger();
    info!("application start");

    // Parse command-line arguments
    let clargs = clargs::parse_command_line_args();

    let config = match app_config::AppConfig::new(&clargs) {
        Ok(config) => config,
        Err(e) => {
            error!("failed to parse command-line arguments: {e}");
            return;
        }
    };

    info!("protocol: {}", config.protocol);
    info!("mode: {}", config.mode);
    info!("remote_url: {}", config.remote_url);
    info!("remote_port: {}", config.remote_port);
    info!("local_port: {}", config.local_port);
    info!("data_payload: {}", config.data_payload);
    info!("count: {}", config.count);
    info!("timeout_in_seconds: {0:.2}", config.timeout_in_seconds);

    let mode = (config.protocol, config.mode);
    match mode {
        (app_config::Protocol::Tcp, app_config::Mode::Client) => {
            tcp::client_task(
                config.remote_url,
                config.remote_port,
                config.local_port,
                config.count,
                config.timeout_in_seconds,
                config.data_payload,
            )
            .await;
        }
        (app_config::Protocol::Tcp, app_config::Mode::Server) => {
            let server_task = tokio::spawn(tcp::server_task(config.local_port));
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
        (app_config::Protocol::Udp, app_config::Mode::Client) => {
            udp::client_task(
                config.remote_url,
                config.remote_port,
                config.local_port,
                config.count,
                config.timeout_in_seconds,
                config.data_payload,
            )
            .await;
        }
        (app_config::Protocol::Udp, app_config::Mode::Server) => {
            let server_task = tokio::spawn(udp::server_task(config.local_port));
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
