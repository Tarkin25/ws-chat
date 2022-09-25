use std::{rc::Rc, cell::RefCell};

use shared::ClientMessage;
use web_sys::HtmlInputElement;
use agents::websocket::{WebSocketAgent, Command};
use yew::prelude::*;
use yew_agent::use_bridge;
use yew_router::prelude::*;

mod components;
mod agents;

use components::AppBar;

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
    let websocket = use_bridge::<WebSocketAgent, _>(|_| {});
    use_effect(move || {
        websocket.send(Command::Open("ws://localhost:8080/websocket".to_string()));

        || {}
    });
    
    html! {
        <ContextProvider<AppContext> context={(*context).clone()}>
            <BrowserRouter>
                <AppBar>
                    {"WS Chat"}
                </AppBar>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </ContextProvider<AppContext>>
     }
}

#[function_component(Join)]
fn join() -> Html {
    let username = use_state(|| String::new());
    let context = use_context::<AppContext>().expect("No context found");
    let websocket = use_bridge::<WebSocketAgent, _>(|_| {});

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
        })
    };
    
    html! {
        <form>
            <input {oninput} placeholder="Username" />
            <Link<Route> to={Route::Chat}>
                <button {onclick} disabled={username.len()<1} >{"Join"}</button>
            </Link<Route>>
        </form>
    }
}

#[function_component(Chat)]
fn chat() -> Html {
    let websocket = use_bridge::<WebSocketAgent, _>(|message| {
        log::info!("{:#?}", message);
    });

    let context = use_context::<AppContext>().expect("Expected AppContext to be available");
    let history = use_history().expect("Expected history to be available");
    use_effect_with_deps(|(context, history)| {
        if context.user.borrow().is_none() {
            history.push(Route::Join);
        }

        || {}
    }, (context, history));

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
        <div>
            <form {onsubmit}>
                <input ref={input_ref} placeholder="Type a message..." />
                <button type="submit">{"Send"}</button>
            </form>
        </div>
    }
}
