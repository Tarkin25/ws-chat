use std::{rc::Rc, cell::RefCell};

use shared::ClientMessage;
use web_sys::HtmlInputElement;
use agents::websocket::{WebSocketAgent, Command};
use yew::prelude::*;
use yew_agent::use_bridge;
use yew_router::prelude::*;

mod components;
mod agents;

use components::{AppBar, Chat};

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
                <div class="p-4 h-screen flex justify-center bg-gray-700">
                    <div class="flex flex-col grow container border rounded border-slate-800">
                        <AppBar>
                            {"WS Chat"}
                        </AppBar>
                        <Switch<Route> render={Switch::render(switch)} />
                    </div>
                </div>
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
