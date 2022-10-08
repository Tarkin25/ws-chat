use std::{rc::Rc, cell::RefCell};

use agents::websocket::{WebSocketAgent, Command};
use yew::prelude::*;
use yew_agent::use_bridge;
use yew_router::prelude::*;

mod components;
mod agents;

use components::{Chat, Join};

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
pub enum Route {
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
                <div class="p-4 h-screen flex justify-center bg-slate-900 text-gray-300">
                    <div class="grow flex flex-col container rounded-lg shadow-lg bg-slate-800">
                        <header class="flex p-4 bg-indigo-800 rounded-tl-lg rounded-tr-lg">
                            <h1 class="text-2xl">{"WS Chat"}</h1>
                        </header>
                        <Switch<Route> render={Switch::render(switch)} />
                    </div>
                </div>
            </BrowserRouter>
        </ContextProvider<AppContext>>
     }
}
