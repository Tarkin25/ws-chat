use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router, Server,
};
use futures::{SinkExt, StreamExt};
use tracing::{instrument, Level};
use tracing_subscriber::FmtSubscriber;

mod stomp;

#[derive(Debug)]
struct AppState {}

#[instrument(skip_all)]
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

#[instrument(skip_all)]
async fn websocket(ws: WebSocket, _state: Arc<AppState>) {
    tracing::debug!("new websocket connection");

    let (mut sender, mut receiver) = ws.split();

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => {
                let echo = format!("Echo: {}", text);
                if let Err(e) = sender.send(Message::Text(echo)).await {
                    tracing::error!("Error while sending: {}", e);
                }
            }
            Message::Close(_) => {
                tracing::debug!("connection closed");
            }
            _ => {}
        }
    }
}

async fn index() -> Html<&'static str> {
    Html("Hello, world!")
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app_state = Arc::new(AppState {});

    let app = Router::new()
        .route("/", get(index))
        .route("/websocket", get(websocket_handler))
        .layer(Extension(app_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
