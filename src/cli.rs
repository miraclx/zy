use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::path::PathBuf;

use clap::{AppSettings, Parser};

macro_rules! DEFAULT_PORT {
    (int) => {
        3000
    };
    (str) => {
        "3000"
    };
}

fn addr_from_str(s: &str) -> Result<SocketAddr, AddrParseError> {
    match s.parse::<u16>() {
        Ok(port) => return Ok(SocketAddr::from(([127, 0, 0, 1], port))),
        Err(_) => {}
    }
    match s.parse::<IpAddr>() {
        Ok(host) => return Ok(SocketAddr::from((host, DEFAULT_PORT!(int)))),
        Err(_) => {}
    }
    s.parse::<SocketAddr>()
}

#[derive(Debug, Parser)]
#[clap(about, version, setting = AppSettings::DeriveDisplayOrder)]
pub struct Args {
    /// Directory to serve
    #[clap(default_value = ".")]
    pub dir: PathBuf,

    /// Sets the address to listen on (repeatable)
    ///
    /// Valid: `3000`, `127.0.0.1`, `127.0.0.1:3000`.
    #[clap(short, long, value_name = "URI", multiple_occurrences = true)]
    #[clap(parse(try_from_str = addr_from_str))]
    #[clap(default_value = concat!("127.0.0.1:", DEFAULT_PORT!(str)))]
    pub listen: Vec<SocketAddr>,

    /// Show debug information
    #[clap(long)]
    pub debug: bool,

    /// Require confirmation before exiting on Ctrl+C
    #[clap(long)]
    pub confirm_exit: bool,
}
