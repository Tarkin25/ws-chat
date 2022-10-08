use std::rc::Rc;

use yew::prelude::*;
use shared::ClientMessage;
use web_sys::HtmlInputElement;
use yew_agent::use_bridge;
use yew_router::prelude::*;

use crate::{AppContext, agents::websocket::{WebSocketAgent, Command}, Route};

#[function_component(Join)]
pub fn join() -> Html {
    let username = use_state(|| String::new());
    let context = use_context::<AppContext>().expect("No context found");
    let websocket = use_bridge::<WebSocketAgent, _>(|_| {});
    let history = use_history().unwrap();

    let oninput = {
        let username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = Rc::clone(&context.user);
        Callback::from(move |_| {
            *user.borrow_mut() = Some((*username).clone());
            websocket.send(Command::SendMessage(ClientMessage::Join((*username).clone())));
            history.push(Route::Chat);
        })
    };
    
    html! {
        <div class="grow flex justify-center items-center">
            <div class="p-4 rounded-lg bg-slate-900 shadow-lg">
                <form class="flex flex-col items-stretch">
                    <h2 class="text-2xl font-bold mb-2">{"Join the chat!"}</h2>
                    <input {oninput} placeholder="Username" class="px-4 py-2 bg-slate-800 rounded focus-visible:outline-none mb-2" />
                    <button {onclick} disabled={username.len()<1} class="p-2 bg-indigo-800 rounded enabled:cursor-pointer disabled:bg-indigo-900 hover:bg-indigo-900" >{"Join"}</button>
                </form>
            </div>
        </div>
    }
}