use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use actix_files as fs;
use actix_web::http::{header, StatusCode};
use actix_web::{guard, web, App, HttpServer};
use actix_web::{HttpRequest, HttpResponse};
use clap::Parser;
use color_eyre::eyre::Result;
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

enum CachePolicy {
    NoCache,
    ShouldCache,
    Undetermined,
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
    let path = normalize_path(Path::new(&*path)).ok()?;

    if let PathSource::Client = source {
        if !state.args.all
            && path
                .file_name()
                .map_or(false, |name| name.to_string_lossy().starts_with('.'))
        {
            return None;
        }
    }

    let mut path = state.args.dir.join(if path.as_os_str().is_empty() {
        state.args.dir.join(&state.args.index)
    } else {
        state.args.dir.join(path).canonicalize().ok()?
    });

    if let PathSource::Client = source {
        if !path.starts_with(&state.args.dir) && !state.args.follow_links {
            return None;
        }
    }

    if path.is_dir() {
        path = path.join("index.html").canonicalize().ok()?;
    }

    if !path.is_file() {
        return None;
    }

    if state.args.verbose {
        debug!(target: "zy::serve", path=%path.strip_prefix(&state.args.dir).ok()?.display());
    }

    let file = fs::NamedFile::open(&path).ok()?;

    let mut res = file
        .use_etag(true)
        .prefer_utf8(true)
        .use_last_modified(true)
        .disable_content_disposition()
        .into_response(&req);

    // https://github.com/GoogleChrome/lighthouse/blob/60c2fa25d11187802e905e4f335b2e7f6df735f1/core/audits/byte-efficiency/uses-long-cache-ttl.js#L144-L164
    if let StatusCode::OK | StatusCode::PARTIAL_CONTENT = res.status() {
        let ext = path.extension().and_then(OsStr::to_str);
        let mime = ext.map_or(mime::APPLICATION_OCTET_STREAM, |ext| {
            actix_files::file_extension_to_mime(ext)
        });

        // https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/devtools/front_end/common/ResourceType.js;l=279-291;drc=3dab2a4e337404c9f853eccb07a0f647004319e2
        let cache_policy = match (mime.type_(), mime.subtype()) {
            (mime::TEXT, mime::HTML) => CachePolicy::NoCache,
            (mime::APPLICATION, mime::JAVASCRIPT)
            | (mime::TEXT, mime::CSS)
            | (mime::IMAGE, _)
            | (mime::TEXT, _)
            | (mime::FONT, _) => CachePolicy::ShouldCache,
            _ if matches!(ext, Some("otf") | Some("woff")) => CachePolicy::ShouldCache,
            _ => CachePolicy::Undetermined,
        };

        let mut cache_directives = vec![];

        match cache_policy {
            CachePolicy::NoCache => {
                cache_directives.extend([
                    header::CacheDirective::NoCache,
                    header::CacheDirective::NoStore,
                ]);
            }
            CachePolicy::ShouldCache => {
                cache_directives.extend([
                    header::CacheDirective::Public,
                    header::CacheDirective::MaxAge(state.args.cache),
                ]);
            }
            CachePolicy::Undetermined => {}
        }

        if let Ok((k, v)) =
            header::TryIntoHeaderPair::try_into_pair(header::CacheControl(cache_directives))
        {
            res.headers_mut().insert(k, v);
        }
    }

    Some(res)
}

async fn index(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<Arc<ServerState>>,
) -> HttpResponse {
    if state.args.verbose {
        debug!(
            target: "zy::request",
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
                    info!(target: "zy::serve", "SPA routing to {}", state.args.index);
                }
                match serve(&req, &state.args.index, PathSource::Server, &state) {
                    Some(res) => return res,
                    None => {}
                }
            }
        }
        if state.args.verbose {
            info!(target: "zy::serve", "not found, serving {}", state.args.not_found);
        }
        match serve(&req, &state.args.not_found, PathSource::Server, &state) {
            Some(mut resp) => {
                *resp.status_mut() = StatusCode::NOT_FOUND;
                resp
            }
            None => {
                if state.args.verbose {
                    info!(target: "zy::serve", "{} not found, omitting response body", state.args.not_found);
                }
                HttpResponse::build(StatusCode::NOT_FOUND).finish()},
        }
    });

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
}

async fn init_app() -> Result<()> {
    let mut args = cli::Args::parse();

    info!("PID: {}", std::process::id());

    if let Ok(port) = env::var("PORT") {
        if let Err(_) = port.parse::<u16>() {
            eprintln!(
                "warning: invalid PORT environment variable: {:?}, ignoring",
                port
            );
        }
    }

    if args.listen.is_empty() {
        args.listen.push(cli::addr_from_str("127.0.0.1").unwrap());
    }

    debug!("Args: {:#?}", args);

    let server_state = Arc::new(ServerState { args });

    let server_state_1 = server_state.clone();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::ZyServer)
            .app_data(web::Data::new(server_state_1.clone()))
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
                .add_directive("zy=info".parse()?)
                .add_directive("zy=debug".parse()?),
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
