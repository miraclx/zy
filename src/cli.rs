use clap::{AppSettings, Parser};

#[derive(Debug, Parser)]
#[clap(author, version, about, setting = AppSettings::DeriveDisplayOrder)]
pub struct Args {
    /// Sets the port to listen on
    #[clap(short, long, default_value = "3000")]
    pub port: u16,
}
