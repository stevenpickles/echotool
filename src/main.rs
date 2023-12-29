use env_logger::Builder;
use log::{error, info, LevelFilter};

mod app_config;
mod clargs;
mod tcp;
mod udp;

#[tokio::main]
async fn main() {
    init_logger();

    // Parse command-line arguments
    let clargs = clargs::parse_command_line_args();

    info!("application start");

    let config = match app_config::AppConfig::new(&clargs) {
        Ok(config) => config,
        Err(e) => {
            error!("failed to parse command-line arguments: {e}");
            return;
        }
    };

    print_config_info(&config);

    run_application(&config).await;

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

fn print_config_info(config: &app_config::AppConfig) {
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
}

async fn run_application(config: &app_config::AppConfig) {
    let mode = (config.protocol, config.mode);
    match mode {
        (app_config::Protocol::Tcp, app_config::Mode::Client) => {
            Box::pin(tcp::client_task(config)).await;
        }
        (app_config::Protocol::Tcp, app_config::Mode::Server) => {
            Box::pin(tcp::server_task(config)).await;
        }
        (app_config::Protocol::Udp, app_config::Mode::Client) => {
            Box::pin(udp::client_task(config)).await;
        }
        (app_config::Protocol::Udp, app_config::Mode::Server) => {
            Box::pin(udp::server_task(config)).await;
        }
    }
}
