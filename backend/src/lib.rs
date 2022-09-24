use std::sync::Arc;

use anyhow::Context;
use axum::extract::ws::{WebSocket, Message};
use futures::StreamExt;
use users::Users;
use shared::{self, ClientMessage};

pub mod users;

#[tracing::instrument(skip(websocket))]
pub async fn handle_connection(mut websocket: WebSocket, users: Arc<Users>) -> anyhow::Result<()> {
    // wait for Join
    let user = loop {
        if let Some(Ok(message)) = websocket.recv().await {
            match message {
                Message::Text(message) => {
                    let message: shared::ClientMessage = serde_json::from_str(&message).context("Unable to deserialize message")?;

                    if let shared::ClientMessage::Join(user) = message {
                        break user;
                    }
                },
                _ => continue
            }
        }
    };

    let (sink, mut stream) = websocket.split();
    tracing::info!("user \"{}\" joined", &user);
    users.join(user.clone(), sink.into()).await.context("Unable to join")?;

    while let Some(Ok(message)) = stream.next().await {
        match message {
            Message::Text(message) => {
                let message: ClientMessage = serde_json::from_str(&message).context("Unable to deserialize message")?;

                if let ClientMessage::SendMessage(message) = message {
                    users.send_message(user.clone(), message).await.context("Unable to send message")?;
                } 
            }
            Message::Close(_) => {
                tracing::info!("user \"{}\" left", &user);
                users.leave(user.clone()).await.context("Unable to leave")?;
            }
            _ => {}
        }
    }
    
    Ok(())
}