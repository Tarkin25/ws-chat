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

use super::{HandleInput, StatefulAgent};

pub struct WebSocketAgent {
    shared: Shared,
    state: State,
}

impl StatefulAgent for WebSocketAgent {
    type State = State;
    type Shared = Shared;
}

pub enum State {
    Initial,
    Open {
        message_sender: Sender<ClientMessage>,
    },
}

pub struct Shared {
    subscribers: Arc<DashSet<HandlerId>>,
    link: AgentLink<WebSocketAgent>,
}

impl HandleInput<WebSocketAgent> for State {
    fn handle_input(
        &mut self,
        input: <WebSocketAgent as Agent>::Input,
        shared: &<WebSocketAgent as StatefulAgent>::Shared,
    ) -> Option<<WebSocketAgent as StatefulAgent>::State> {
        match self {
            State::Initial => {
                if let Command::Open(url) = input {
                    Some(Self::open(url, shared))
                } else {
                    None
                }
            }
            State::Open { message_sender } => {
                if let Command::SendMessage(message) = input {
                    message_sender
                        .try_send(message)
                        .expect("Unable to send message");
                }

                None
            }
        }
    }
}

impl State {
    fn open(url: String, shared: &Shared) -> Self {
        let websocket = WebSocket::open(&url).expect("Unable to open websocket");
        let (websocket_sender, websocket_receiver) = websocket.split();
        let (message_sender, message_receiver) = futures::channel::mpsc::channel(1000);

        spawn_local(read_messages_from_websocket(
            websocket_receiver,
            shared.link.clone(),
            Arc::clone(&shared.subscribers),
        ));

        spawn_local(send_messages_to_websocket(
            message_receiver,
            websocket_sender,
        ));

        Self::Open { message_sender }
    }
}

pub enum Command {
    Open(String),
    SendMessage(ClientMessage),
}

impl Agent for WebSocketAgent {
    type Message = ();
    type Reach = yew_agent::Context<Self>;
    type Input = Command;
    type Output = ServerMessage;

    fn create(link: AgentLink<Self>) -> Self {
        let subscribers = Arc::new(DashSet::default());

        Self {
            shared: Shared { subscribers, link },
            state: State::Initial,
        }
    }

    fn handle_input(&mut self, message: Self::Input, _id: HandlerId) {
        if let Some(new_state) = self.state.handle_input(message, &self.shared) {
            self.state = new_state;
        }
    }

    fn connected(&mut self, id: yew_agent::HandlerId) {
        self.shared.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.shared.subscribers.remove(&id);
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
            let message: ServerMessage =
                serde_json::from_str(&message).expect("Unable to deserialize server message");

            subscribers.iter().for_each(|subscriber| {
                link.respond(*subscriber, message.clone());
            });
        }
    }
}
