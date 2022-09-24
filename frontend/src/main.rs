use std::{rc::Rc, cell::RefCell};

use shared::ClientMessage;
use web_sys::HtmlInputElement;
use websocket::WebSocketAgent;
use yew::prelude::*;
use yew_agent::use_bridge;
use yew_router::prelude::*;

mod websocket;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Join => html! { <Join />},
        Route::Chat => html! { <Chat /> },
        Route::NotFound => html! { <p>{"Not Found"}</p>}
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Join,
    #[at("/chat")]
    Chat,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AppContext {
    user: Rc<RefCell<Option<String>>>,
}

#[function_component(App)]
fn app() -> Html {
    let context = use_state(AppContext::default);
    
    html! {
        <ContextProvider<AppContext> context={(*context).clone()}>
            <BrowserRouter>
                <h1 class="text-3xl font-bold underline">
                    {"WS Chat"}
                </h1>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </ContextProvider<AppContext>>
     }
}

#[function_component(Join)]
fn join() -> Html {
    let username = use_state(|| String::new());
    let context = use_context::<AppContext>().expect("No context found");
    let websocket = use_bridge::<WebSocketAgent, _>(|message| {
        log::info!("Received {:#?}", message);
    });

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = Rc::clone(&context.user);
        Callback::from(move |_| {
            *user.borrow_mut() = Some((*username).clone());
            websocket.send(ClientMessage::Join((*username).clone()));
        })
    };
    
    html! {
        <div class="bg-gray-800 flex w-screen">
            <div class="container mx-auto flex flex-col justify-center items-center">
                <form class="m-4 flex">
                    <input {oninput} placeholder="Username" class="rounded-l-lg p-4 border-t mr-0 border-b border-l text-gray-800 border-gray-200 bg-white" />
                    <Link<Route> to={Route::Chat}>
                        <button {onclick} disabled={username.len()<1} class="px-8 rounded-r-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r">{"Join"}</button>
                    </Link<Route>>
                </form>
            </div>
        </div>
    }
}

#[function_component(Chat)]
fn chat() -> Html {
    let websocket = use_bridge::<WebSocketAgent, _>(|message| {
        log::info!("Received {:#?}", message);
    });

    let input_ref = use_node_ref();

    let onsubmit = {
        let input_ref = input_ref.clone();
        let websocket = websocket.clone();
        
        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let input = input_ref.cast::<HtmlInputElement>().unwrap();
            websocket.send(ClientMessage::SendMessage(input.value()));
            input.set_value("");
        })
    };
    
    html! {
        <div>
            <form {onsubmit}>
                <input ref={input_ref} placeholder="Type a message..." />
                <button type="submit">{"Send"}</button>
            </form>
        </div>
    }
}
