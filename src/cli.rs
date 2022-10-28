use std::path::PathBuf;

use clap::{AppSettings, Parser};

#[derive(Debug, Parser)]
#[clap(about, version, setting = AppSettings::DeriveDisplayOrder)]
pub struct Args {
    /// Directory to serve
    #[clap(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Sets the port to listen on
    #[clap(short, long, default_value = "3000")]
    pub port: u16,

    /// Sets the address to listen on
    #[clap(short, long, default_value_t = [127, 0, 0, 1].into())]
    pub host: std::net::IpAddr,
}
