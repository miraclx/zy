use axum::{body::Body, routing::get, Router};

async fn root() -> &'static str {
    "Hello, World!"
}

pub fn load() -> Router<Body> {
    Router::new().route("/", get(root))
}
