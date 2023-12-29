use clap::ArgMatches;
use core::fmt;
use std::error;

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp => write!(f, "TCP"),
            Self::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Client,
    Server,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Client => write!(f, "client"),
            Self::Server => write!(f, "server"),
        }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub mode: Mode,
    pub remote_url: String,
    pub remote_port: u16,
    pub local_port: u16,
    pub data_payload: String,
    pub count: u32,
    pub protocol: Protocol,
    pub timeout_in_seconds: f64,
}

impl AppConfig {
    pub fn new(clargs: &ArgMatches) -> Result<Self, Box<dyn error::Error>> {
        let is_client_mode = clargs.contains_id("remote_url");
        let mode = if is_client_mode {
            Mode::Client
        } else {
            Mode::Server
        };

        let remote_url = match mode {
            Mode::Client => match clargs.get_one::<String>("remote_url") {
                Some(value) => value,
                None => return Err("remote_url is invalid".into()),
            },
            Mode::Server => "<none>",
        };

        let Some(remote_port) = clargs.get_one::<u16>("remote_port") else {
            return Err("remote_port is invalid".into());
        };

        let Some(local_port) = clargs.get_one::<u16>("local_port") else {
            return Err("local_port is invalid".into());
        };

        let Some(data_payload) = clargs.get_one::<String>("data_payload") else {
            return Err("data_payload is invalid".into());
        };

        let Some(count) = clargs.get_one::<u32>("count") else {
            return Err("count is invalid".into());
        };

        let Some(timeout_in_seconds) = clargs.get_one::<f64>("timeout_in_seconds") else {
            return Err("timeout_in_seconds is invalid".into());
        };

        let protocol = match clargs.get_one::<String>("protocol") {
            Some(value) => match value.as_str() {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                _ => return Err("protocol is invalid".into()),
            },
            None => return Err("protocol is invalid".into()),
        };

        Ok(Self {
            mode,
            remote_url: remote_url.to_string(),
            remote_port: *remote_port,
            local_port: *local_port,
            data_payload: data_payload.to_string(),
            count: *count,
            protocol,
            timeout_in_seconds: *timeout_in_seconds,
        })
    }
}
