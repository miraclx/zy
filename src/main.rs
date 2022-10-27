use std::sync::Arc;

use axum::Extension;
use clap::Parser;
use color_eyre::eyre::Result;
use tokio::sync::mpsc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod cli;
mod exit;
mod routes;

pub struct ServerState {
    // port: u16,
    shutdown_signal: mpsc::Sender<()>,
}

async fn init_app() -> Result<()> {
    let args = cli::Args::parse();

    info!("PID: {}", std::process::id());

    debug!("Args: {:#?}", args);

    let (shutdown_tx, mut shutdown_signal) = mpsc::channel::<()>(1);

    let server_state = Arc::new(ServerState {
        // port: args.port,
        shutdown_signal: shutdown_tx,
    });

    let app = routes::load()
        .layer(Extension(server_state.clone()))
        .into_make_service();

    let server = axum::Server::bind(&([0; 4], args.port).into())
        .serve(app)
        .with_graceful_shutdown(async {
            shutdown_signal.recv().await;
            info!("Starting Graceful Shutdown");
        });

    tokio::select! {
        _ = server => {}
        _ = exit::on_exit(&server_state.shutdown_signal) => {}
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

#[tokio::main]
async fn main() -> Result<()> {
    setup()?;

    init_app().await?;

    Ok(())
}
