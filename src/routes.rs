use std::sync::Arc;

use axum::body::Body;
use axum::routing::Router;

use super::ServerState;

mod service {
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
            .layer(CompressionLayer::new().gzip(true))
            .route("/ping", get(|| async { "pong" }))
            .fallback(serve_dir_service)
            .layer(Extension(state))
    }
}

mod handler {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::extract::Extension;
    use axum::http::{Request, StatusCode};
    use axum::response::Response;
    use axum::routing::{get, Router};
    use tower::ServiceExt;
    use tower_http::compression::CompressionLayer;
    use tower_http::services::fs::ServeFileSystemResponseBody;
    use tower_http::services::ServeDir;

    use super::ServerState;

    pub fn load(state: Arc<ServerState>) -> Router<Body> {
        Router::new()
            .layer(CompressionLayer::new().gzip(true))
            .route("/ping", get(|| async { "pong" }))
            .fallback(Router::new().route("/*path", get(handler)))
            .layer(Extension(state))
    }

    async fn handler(
        Extension(state): Extension<Arc<ServerState>>,
        request: Request<Body>,
    ) -> Result<Response<ServeFileSystemResponseBody>, (StatusCode, String)> {
        ServeDir::new(&state.dir)
            .oneshot(request)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", err),
                )
            })
    }
}

pub fn load(state: Arc<ServerState>) -> Router<Body> {
    if true {
        service::load(state.clone())
    } else {
        handler::load(state.clone())
    }
}
