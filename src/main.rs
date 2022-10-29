use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use actix_files as fs;
use actix_web::http::{header, StatusCode};
use actix_web::{guard, web, App, HttpServer};
use actix_web::{HttpRequest, HttpResponse};
use clap::Parser;
use color_eyre::eyre::Result;
#[cfg(feature = "shutdown-signal")]
use tokio::sync::mpsc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod cli;
mod exit;
mod middleware;

// /a => a
// a/../b => b
// /a/b/../c/./d => a/c/d
// C:\a => ERROR
fn normalize_path<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let mut buf = PathBuf::new();
    for c in path.as_ref().components() {
        match c {
            Component::Normal(c) => buf.push(c),
            Component::ParentDir => {
                buf.pop();
            }
            Component::CurDir | Component::RootDir => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "prefix is not supported",
                ))
            }
        }
    }
    for c in buf.components() {
        assert!(matches!(c, Component::Normal(_)));
    }
    Ok(buf)
}

enum PathSource {
    Client,
    Server,
}

fn serve(
    req: &HttpRequest,
    path: &str,
    source: PathSource,
    state: &ServerState,
) -> Option<HttpResponse> {
    let mut source = source;
    let path = normalize_path(Path::new(&*path)).ok()?;

    let path = if path.as_os_str().is_empty() {
        source = PathSource::Server;
        Path::new(&state.args.index)
    } else {
        &path
    };
    let path = state.args.dir.join(path);

    if state.args.verbose {
        debug!(target: "mythian::serve", path=%path.display());
    }

    if let PathSource::Client = source {
        if !state.args.all && path.file_name()?.to_string_lossy().starts_with('.') {
            return None;
        }

        if !state.args.follow_links && path.is_symlink() {
            return None;
        }
    }

    let file = fs::NamedFile::open(path).ok()?;

    Some(
        file.disable_content_disposition()
            .prefer_utf8(true)
            .into_response(&req),
    )
}

async fn index(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<Arc<ServerState>>,
) -> HttpResponse {
    if state.args.verbose {
        debug!(
            target: "mythian::request",
            version = ?req.version(),
            method = %req.method(),
            uri = %req.uri(),
        );
    }

    let mut res = serve(&req, &path, PathSource::Client, &state).unwrap_or_else(|| {
        if state.args.spa {
            let accepts_html = <header::Accept as header::Header>::parse(&req)
                .map_or(false, |accept| {
                    accept.iter().any(|mime| mime.item == "text/html")
                });
            if accepts_html {
                if state.args.verbose {
                    info!(target: "mythian::serve", "spa routing to {}", state.args.index);
                }
                match serve(&req, &state.args.index, PathSource::Server, &state) {
                    Some(res) => return res,
                    None => {}
                }
            }
        }
        if state.args.verbose {
            info!(target: "mythian::serve", "not found, serving {}", state.args.not_found);
        }
        match serve(&req, &state.args.not_found, PathSource::Server, &state) {
            Some(mut resp) => {
                *resp.status_mut() = StatusCode::NOT_FOUND;
                resp
            }
            None => HttpResponse::build(StatusCode::NOT_FOUND).finish(),
        }
    });

    if let Ok((k, v)) = header::TryIntoHeaderPair::try_into_pair(header::CacheControl(vec![
        header::CacheDirective::Public,
        header::CacheDirective::MaxAge(state.args.cache),
    ])) {
        res.headers_mut().insert(k, v);
    }

    if !state.args.no_cors {
        res.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::HeaderValue::from_static("*"),
        );
    }

    res
}

pub struct ServerState {
    args: cli::Args,
    #[cfg(feature = "shutdown-signal")]
    shutdown_signal: mpsc::Sender<()>,
}

async fn init_app() -> Result<()> {
    let args = cli::Args::parse();

    info!("PID: {}", std::process::id());

    debug!("Args: {:#?}", args);

    #[cfg(feature = "shutdown-signal")]
    let (shutdown_tx, mut shutdown_signal) = mpsc::channel(1);

    let server_state = Arc::new(ServerState {
        args: args,
        #[cfg(feature = "shutdown-signal")]
        shutdown_signal: shutdown_tx,
    });

    let server_state_1 = server_state.clone();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::MythianServer)
            .app_data(web::Data::new(server_state_1.clone()))
            .service(
                web::resource("/ping")
                    .guard(guard::Any(guard::Get()).or(guard::Head()))
                    .to(|| async { "pong" }),
            )
            .service(
                web::resource("/{path:.*}")
                    .guard(guard::Any(guard::Get()).or(guard::Head()))
                    .wrap(middleware::Compress::default())
                    .to(index),
            )
    })
    .disable_signals();

    for addr in &server_state.args.listen {
        server = server.bind(addr)?;
        info!("Listening on http://{}", addr);
    }

    let server = server.run();

    let server_handle = server.handle();

    tokio::select! {
        _ = server => {}
        _ = exit::on_signal(
            server_state.args.confirm_exit,
            #[cfg(feature = "shutdown-signal")] &mut shutdown_signal,
            |graceful| async move {
                if graceful {
                    info!("Starting graceful shutdown");
                } else {
                    info!("Shutting down immediately");
                }
                server_handle.stop(graceful).await;
            }
        ) => {}
    }

    Ok(())
}

fn setup() -> Result<()> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("mythian=info".parse()?)
                .add_directive("mythian=debug".parse()?),
        )
        .init();

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    setup()?;

    init_app().await?;

    Ok(())
}
