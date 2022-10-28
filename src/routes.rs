use std::io;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::routing::{get, get_service, Router};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use super::ServerState;

pub fn load(state: Arc<ServerState>) -> Router<Body> {
    let serve_dir_service =
        get_service(ServeDir::new(&state.dir)).handle_error(|error: io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        });

    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .fallback(serve_dir_service)
        .layer(Extension(state))
        .layer(CompressionLayer::new().gzip(true))
}
