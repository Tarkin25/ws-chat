use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::WebSocket,
        WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Extension, Router, Server,
};
use backend::users::Users;
use backend::handle_connection;
use tracing::{instrument, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug)]
struct AppState {}

#[instrument(skip_all)]
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(users): Extension<Arc<Users>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, users))
}

#[instrument(skip_all)]
async fn websocket(ws: WebSocket, users: Arc<Users>) {
    tracing::debug!("new websocket connection");

    if let Err(error) = handle_connection(ws, users).await {
        tracing::error!("{}", error);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let users = Arc::new(Users::default());

    let app = Router::new()
        .route("/websocket", get(websocket_handler))
        .layer(Extension(users));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
