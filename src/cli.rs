use std::ffi::OsStr;
use std::io;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::path::PathBuf;

use clap::{AppSettings, Parser};

#[macro_export]
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

fn parse_canonicalize_dir(s: &OsStr) -> Result<PathBuf, io::Error> {
    PathBuf::from(s).canonicalize()
}

#[derive(Debug, Parser)]
#[clap(name = "Zy")]
#[clap(about, version, setting = AppSettings::DeriveDisplayOrder)]
#[clap(after_help = "The PORT environment variable is also supported.")]
pub struct Args {
    /// Directory to serve
    #[clap(default_value = ".", parse(try_from_os_str = parse_canonicalize_dir))]
    pub dir: PathBuf,

    /// Sets the address to listen on (repeatable)
    /// Valid: `3000`, `127.0.0.1`, `127.0.0.1:3000` [default: 127.0.0.1:3000]
    #[clap(short, long, value_name = "URI", multiple_occurrences = true)]
    #[clap(verbatim_doc_comment, parse(try_from_str = addr_from_str))]
    pub listen: Vec<SocketAddr>,

    /// Run as a Single Page Application
    #[clap(short, long)]
    pub spa: bool,

    /// Index file to serve from the base directory.
    #[clap(short, long, value_name = "FILE", default_value = "index.html")]
    pub index: String,

    /// 404 file to serve from the base directory.
    #[clap(long = "404", value_name = "FILE", default_value = "404.html")]
    pub not_found: String,

    /// Cache time (max-age) in seconds
    #[clap(short, long, value_name = "SECS", default_value_t = 3600)]
    pub cache: u32,

    /// Disable Cross-Origin Resource Sharing (CORS)
    #[clap(long)]
    pub no_cors: bool,

    /// Serve hidden files
    #[clap(short, long)]
    pub all: bool,

    /// Follow symlinks outside of the base directory (unsafe)
    #[clap(short, long)]
    pub follow_links: bool,

    /// Be verbose
    #[clap(short, long)]
    pub verbose: bool,

    /// Require confirmation before exiting on Ctrl+C
    #[clap(short = 'x', long)]
    pub confirm_exit: bool,
}
