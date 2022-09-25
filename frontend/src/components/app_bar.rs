use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct AppBarProps {
    pub children: Children,
}

#[function_component(AppBar)]
pub fn app_bar(props: &AppBarProps) -> Html {
    
    html! {
        <header class="flex p-4 bg-purple-700">
            <h1 class="text-2xl text-white">{props.children.clone()}</h1>
        </header>
    }
}