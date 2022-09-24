use yew::prelude::*;

fn main() {
    yew::start_app::<HelloWorld>();
}

#[function_component(HelloWorld)]
fn hello_world() -> Html {
    html! { 
        <h1 class="text-3xl font-bold underline">
            {"Hello World!"}
        </h1>
     }
}
