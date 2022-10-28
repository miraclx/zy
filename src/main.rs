use std::path::{Path, PathBuf};
use std::sync::Arc;

use actix_files as fs;
use actix_web::dev::Service;
use actix_web::http::{header, StatusCode};
use actix_web::{middleware, web, App, HttpServer};
use actix_web::{HttpRequest, HttpResponse};
use clap::Parser;
use color_eyre::eyre::Result;
// use tokio::sync::mpsc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod cli;
mod exit;

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

fn not_found() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/plain; charset=utf-8")
        .body("Not Found")
}

fn serve(req: &HttpRequest, path: &Path) -> HttpResponse {
    let file = match fs::NamedFile::open(path) {
        Ok(file) => file,
        Err(_) => return not_found(),
    };

    file.disable_content_disposition()
        .prefer_utf8(true)
        .into_response(&req)
}

async fn index(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<Arc<ServerState>>,
) -> HttpResponse {
    let path = match normalize_path(Path::new(&*path)) {
        Ok(path) => path,
        Err(_) => return not_found(),
    };

    let path = state.dir.join(if path.as_os_str().is_empty() {
        Path::new("index.html")
    } else {
        &path
    });

    serve(&req, &path)
}

#[derive(Clone)]
pub struct ServerState {
    dir: PathBuf,
    // shutdown_signal: mpsc::Sender<()>,
}

async fn init_app() -> Result<()> {
    let args = cli::Args::parse();

    info!("PID: {}", std::process::id());

    debug!("Args: {:#?}", args);

    // let (shutdown_tx, mut shutdown_signal) = mpsc::channel(1);

    let server_state = Arc::new(ServerState {
        dir: args.dir.canonicalize()?,
        // shutdown_signal: shutdown_tx,
    });

    let server = HttpServer::new(move || {
        App::new()
            .route("/ping", web::get().to(|| async { "pong" }))
            .route("/{filename:.*}", web::get().to(index))
            .app_data(web::Data::new(server_state.clone()))
            .wrap(middleware::Compress::default())
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    res.headers_mut()
                        .insert(header::SERVER, header::HeaderValue::from_static("Mythian"));
                    res.headers_mut().insert(
                        header::CACHE_CONTROL,
                        header::HeaderValue::from_static("public, max-age=3600"),
                    );
                    Ok(res)
                }
            })
        // .service(
        //     fs::Files::new("/", &args.dir)
        //         .index_file("index.html")
        //         .prefer_utf8(true)
        //         // .path_filter(|path, |)
        //         .disable_content_disposition()
        //         .use_hidden_files()
        //         .show_files_listing(),
        // )
    })
    .bind((args.host, args.port))?
    .run();

    info!("Listening on http://{}:{}", args.host, args.port);
    tokio::select! {
        _ = server => {}
        _ = exit::on_signal() => {}
        // _ = shutdown_signal.recv() => {}
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
                .add_directive("bulan_server=info".parse()?)
                .add_directive("bulan_server=debug".parse()?),
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
