use serde::{Serialize, Deserialize};

use crate::error::Error;

#[derive(Debug, Serialize, Clone)]
pub enum ServerMessage {
    Joined(String),
    Left(String),
    MessageSent {
        user: String,
        message: String,
    },
    Error(Error),
}

#[derive(Debug, Deserialize)]
pub enum ClientMessage {
    Join(String),
    SendMessage(String)
}