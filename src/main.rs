use env_logger::Builder;
use log::{error, info, LevelFilter};

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

    match config.mode {
        app_config::Mode::Client => {
            info!("remote_url: {}", config.remote_url);
            info!("remote_port: {}", config.remote_port);
            info!("local_port: {}", config.local_port);
            info!("data_payload: {}", config.data_payload);
            info!("count: {}", config.count);
            info!("timeout_in_seconds: {0:.2}", config.timeout_in_seconds);
        }
        app_config::Mode::Server => {
            info!("local_port: {}", config.local_port);
        }
    }

    let mode = (config.protocol, config.mode);
    match mode {
        (app_config::Protocol::Tcp, app_config::Mode::Client) => {
            Box::pin(tcp::client_task(&config)).await;
        }
        (app_config::Protocol::Tcp, app_config::Mode::Server) => {
            Box::pin(tcp::server_task(&config)).await;
        }
        (app_config::Protocol::Udp, app_config::Mode::Client) => {
            Box::pin(udp::client_task(&config)).await;
        }
        (app_config::Protocol::Udp, app_config::Mode::Server) => {
            Box::pin(udp::server_task(&config)).await;
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
