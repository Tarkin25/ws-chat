use serde::{Serialize, Deserialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    Joined(String),
    Left(String),
    MessageSent {
        user: String,
        message: String,
    },
    Error(Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    Join(String),
    SendMessage(String)
}