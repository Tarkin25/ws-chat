use std::sync::Arc;

use dashmap::DashSet;
use futures::{
    channel::mpsc::{Receiver, Sender},
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use reqwasm::websocket::{futures::WebSocket, Message};
use shared::{ClientMessage, ServerMessage};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, HandlerId};

pub struct WebSocketAgent {
    subscribers: Arc<DashSet<HandlerId>>,
    message_sender: Sender<ClientMessage>,
}

impl Agent for WebSocketAgent {
    type Message = ();
    type Reach = yew_agent::Context<Self>;
    type Input = ClientMessage;
    type Output = ServerMessage;

    fn create(link: AgentLink<Self>) -> Self {
        let websocket =
            WebSocket::open("ws://localhost:8080/websocket").expect("Unable to open websocket");
        let (websocket_sender, websocket_receiver) = websocket.split();
        let (message_sender, message_receiver) = futures::channel::mpsc::channel(1000);
        let subscribers = Arc::new(DashSet::default());

        spawn_local(read_messages_from_websocket(
            websocket_receiver,
            link,
            Arc::clone(&subscribers)
        ));

        spawn_local(send_messages_to_websocket(
            message_receiver,
            websocket_sender,
        ));

        Self {
            subscribers,
            message_sender,
        }
    }

    fn handle_input(&mut self, message: Self::Input, _id: HandlerId) {
        let mut message_sender = self.message_sender.clone();
        spawn_local(async move {
            message_sender.send(message).await.expect("Unable to send message to self.message_receiver");
        });
    }

    fn connected(&mut self, id: yew_agent::HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn update(&mut self, _msg: Self::Message) {}
}

async fn send_messages_to_websocket(
    mut message_receiver: Receiver<ClientMessage>,
    mut websocket_sender: SplitSink<WebSocket, Message>,
) {
    while let Some(message) = message_receiver.next().await {
        let message = serde_json::to_string(&message).expect("Unable to serialize client message");
        websocket_sender
            .send(Message::Text(message))
            .await
            .expect("Unable to send message");
    }
}

async fn read_messages_from_websocket(
    mut websocket_receiver: SplitStream<WebSocket>,
    link: AgentLink<WebSocketAgent>,
    subscribers: Arc<DashSet<HandlerId>>,
) {
    while let Some(Ok(message)) = websocket_receiver.next().await {
        if let Message::Text(message) = message {
            let message: ServerMessage = serde_json::from_str(&message).expect("Unable to deserialize server message");

            subscribers.iter().for_each(|subscriber| {
                link.respond(*subscriber, message.clone());
            });
        }
    }
}
