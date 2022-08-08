use std::{sync::{Arc, atomic::AtomicUsize}, net::SocketAddr};

use axum::{Router, routing::get, Extension, http::{StatusCode, HeaderMap}, Server};

#[derive(Clone, Default)]
struct AppState {
    counter: Arc<AtomicUsize>,
}

async fn home(state: Extension<AppState>) -> (StatusCode, HeaderMap, String) {
    let counter = state.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/plain; charset=utf-8".parse().unwrap());
    let body = format!("Counter is at: {}", counter);
    (StatusCode::OK, headers, body)
}

#[tokio::main]
async fn main() {
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = Router::new()
        .route("/", get(home))
        .layer(Extension(AppState::default()));

    let server = Server::bind(&address).serve(app.into_make_service());

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
