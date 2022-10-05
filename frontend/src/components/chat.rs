use shared::{ClientMessage, ServerMessage};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use yew_agent::use_bridge;

use crate::{agents::websocket::{WebSocketAgent, Command}, AppContext, Route};
use crate::components::{JoinedMessage, LeftMessage, SentMessage};

#[function_component(Chat)]
pub fn chat() -> Html {
    let messages = use_state(|| Vec::new());
    
    let websocket = {
        let messages = messages.clone();
        
        use_bridge::<WebSocketAgent, _>(move |message| {
            log::info!("{:#?}", message);
            let mut new_messages = (*messages).clone();
            new_messages.push(message);
            messages.set(new_messages);
        })
    };

    require_user();

    let input_ref = use_node_ref();

    let onsubmit = {
        let input_ref = input_ref.clone();
        let websocket = websocket.clone();
        
        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let input = input_ref.cast::<HtmlInputElement>().unwrap();
            websocket.send(Command::SendMessage(ClientMessage::SendMessage(input.value())));
            input.set_value("");
        })
    };
    
    html! {
        <div class="grow flex flex-col">
            <div class="grow">
                {
                    messages.iter().cloned().map(|message| match message {
                        ServerMessage::Joined(user) => html! { <JoinedMessage {user} /> },
                        ServerMessage::Left(user) => html! { <LeftMessage {user} /> },
                        ServerMessage::MessageSent { user, message } => html! { <SentMessage {user} {message} /> },
                        _ => html! {}
                    }).collect::<Html>()
                }
            </div>
            <form {onsubmit}>
                <input ref={input_ref} placeholder="Type a message..." />
                <button type="submit">{"Send"}</button>
            </form>
        </div>
    }
}

fn require_user() {
    let context = use_context::<AppContext>().expect("Expected AppContext to be available");
    let history = use_history().expect("Expected history to be available");
    use_effect_with_deps(|(context, history)| {
        if context.user.borrow().is_none() {
            history.push(Route::Join);
        }

        || {}
    }, (context, history));
}