use std::path::{Path, PathBuf};
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
fn normalize_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ()> {
    let mut buf = PathBuf::new();
    for c in path.as_ref().components() {
        match c {
            std::path::Component::Normal(c) => buf.push(c),
            std::path::Component::ParentDir => {
                buf.pop();
            }
            std::path::Component::CurDir | std::path::Component::RootDir => {}
            _ => return Err(()),
        }
    }
    for c in buf.components() {
        assert!(matches!(c, std::path::Component::Normal(_)));
    }
    Ok(buf)
}

fn serve(req: &HttpRequest, path: &str, state: &ServerState) -> Option<HttpResponse> {
    let path = match normalize_path(Path::new(&*path)) {
        Ok(path) => path,
        Err(_) => return None,
    };

    let path = state.dir.join(if path.as_os_str().is_empty() {
        Path::new("index.html")
    } else {
        &path
    });

    #[cfg(debug_assertions)]
    debug!(target: "mythian::serve", path=%path.display());

    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(_) => return None,
    };

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
    #[cfg(debug_assertions)]
    debug!(
        target: "mythian::request",
        version = ?req.version(),
        method = %req.method(),
        uri = %req.uri(),
    );

    let mut res = serve(&req, &path, &state).unwrap_or_else(|| {
        #[cfg(debug_assertions)]
        info!(target: "mythian::serve", "serving 404.html");
        match serve(&req, "404.html", &state) {
            Some(mut resp) => {
                *resp.status_mut() = StatusCode::NOT_FOUND;
                resp
            }
            None => HttpResponse::build(StatusCode::NOT_FOUND).finish(),
        }
    });

    res.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=3600"),
    );

    res
}

pub struct ServerState {
    dir: PathBuf,
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
        dir: args.dir.canonicalize()?,
        #[cfg(feature = "shutdown-signal")]
        shutdown_signal: shutdown_tx,
    });

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::MythianServer)
            .app_data(web::Data::new(server_state.clone()))
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

    for addr in args.listen {
        server = server.bind(addr)?;
        info!("Listening on http://{}", addr);
    }

    let server = server.run();

    let server_handle = server.handle();

    tokio::select! {
        _ = server => {}
        _ = exit::on_signal(
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
