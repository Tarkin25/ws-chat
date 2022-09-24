use std::fmt::Debug;

use anyhow::Context;
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::{stream::SplitSink, SinkExt};

use shared::{ServerMessage, Error};

#[derive(Debug, Default)]
pub struct Users(DashMap<String, ServerSender>);

impl Users {
    pub async fn join(
        &self,
        username: String,
        mut sender: ServerSender,
    ) -> anyhow::Result<()> {
        if self.0.contains_key(&username) {
            sender
                .send(ServerMessage::Error(Error::UsernameTaken))
                .await
        } else {
            self.0.insert(username.clone(), sender);
            self.broadcast(ServerMessage::Joined(username)).await
        }
    }

    pub async fn leave(
        &self,
        username: String,
    ) -> anyhow::Result<()> {
        if self.0.remove(&username).is_some() {
            self.broadcast(ServerMessage::Left(username)).await
        } else {
            Ok(())
        }
    }

    pub async fn send_message(
        &self,
        user: String,
        message: String,
    ) -> anyhow::Result<()> {
        self.broadcast(ServerMessage::MessageSent { user, message }).await
    }

    async fn broadcast(&self, message: ServerMessage) -> anyhow::Result<()> {
        for mut entry in self.0.iter_mut() {
            entry.value_mut().send(message.clone()).await?;
        }

        Ok(())
    }
}

pub struct ServerSender(SplitSink<WebSocket, Message>);

impl Debug for ServerSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerSender").finish()
    }
}

impl ServerSender {
    pub async fn send(&mut self, message: ServerMessage) -> anyhow::Result<()> {
        let json_string = serde_json::to_string(&message).context("Unable to serialize message")?;
        self.0
            .send(Message::Text(json_string))
            .await
            .context("Unable to send message")
    }
}

impl From<SplitSink<WebSocket, Message>> for ServerSender {
    fn from(sink: SplitSink<WebSocket, Message>) -> Self {
        Self(sink)
    }
}
