use std::env;
use std::ffi::OsStr;
use std::io;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::num::{FpCategory, IntErrorKind};
use std::path::PathBuf;

use clap::{AppSettings, Parser};
use humantime::{parse_duration, DurationError};

const DEFAULT_PORT: u16 = 3000;

pub fn addr_from_str(s: &str) -> Result<SocketAddr, AddrParseError> {
    match s.parse::<u16>() {
        Ok(port) => return Ok(SocketAddr::from(([127, 0, 0, 1], port))),
        Err(_) => {}
    }
    match s.parse::<IpAddr>() {
        Ok(host) => {
            return Ok(SocketAddr::from((
                host,
                env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(DEFAULT_PORT),
            )))
        }
        Err(_) => {}
    }
    s.parse::<SocketAddr>()
}

#[derive(Debug)]
pub struct CanonicalizedPath {
    pub raw: PathBuf,
    pub canonical: PathBuf,
}

impl CanonicalizedPath {
    pub fn is_current_dir(&self) -> bool {
        env::current_dir().map_or(false, |cwd| cwd == self.canonical)
    }
}

fn parse_canonicalize_dir(s: &OsStr) -> Result<CanonicalizedPath, io::Error> {
    let raw = PathBuf::from(s);
    let canonical = raw.canonicalize()?;
    Ok(CanonicalizedPath { raw, canonical })
}

fn parse_cache_time(s: &str) -> color_eyre::Result<u32> {
    let duration = match parse_duration(s) {
        Ok(duration) => duration,
        Err(err) => {
            if matches!(
                err,
                DurationError::UnknownUnit { ref unit, .. }
                if unit.is_empty()
            ) {
                match s.parse::<u32>() {
                    Ok(seconds) => return Ok(seconds),
                    Err(err) if *err.kind() == IntErrorKind::PosOverflow => {
                        return Err(color_eyre::eyre::eyre!("cache time is too large"))
                    }
                    Err(_) => {}
                }
            }
            return Err(err.into());
        }
    };
    let secs = duration.as_secs_f64();
    assert!(
        matches!(secs.classify(), FpCategory::Normal | FpCategory::Zero),
        "humantime should not return NaN or infinite values"
    );
    if secs != secs.trunc() {
        return Err(color_eyre::eyre::eyre!(
            "cache time in seconds cannot be fractional"
        ));
    }
    if secs > u32::MAX as f64 {
        return Err(color_eyre::eyre::eyre!("cache time is too large"));
    }
    Ok(secs as u32)
}

#[derive(Debug, Parser)]
#[clap(name = "Zy")]
#[clap(about, version, setting = AppSettings::DeriveDisplayOrder)]
#[clap(after_help = "The PORT environment variable is also supported.")]
pub struct Args {
    /// Directory to serve
    #[clap(default_value = ".", parse(try_from_os_str = parse_canonicalize_dir))]
    pub dir: CanonicalizedPath,

    /// Sets the address to listen on (repeatable) [default: 127.0.0.1:3000]
    /// Valid: `3000`, `127.0.0.1`, `127.0.0.1:3000` [env: PORT]
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

    /// Cache time (max-age) [default: 1h]
    /// Valid: `10` for 10 seconds, `1h`, `1year 6months`
    #[clap(short, long, value_name = "TIME", verbatim_doc_comment)]
    #[clap(default_value = "1h", hide_default_value = true)]
    #[clap(parse(try_from_str = parse_cache_time))]
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

    /// Hide the `Server` and `X-Powered-By` headers [alias: `--anon`]
    #[clap(short = 'Z', long, alias = "anon")]
    pub anonymize: bool,
}
