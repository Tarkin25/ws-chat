use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct JoinedProps {
    pub user: String
}

#[function_component(JoinedMessage)]
pub fn joined_message(props: &JoinedProps) -> Html {
    html! {
        <div class="flex justify-center">
            <p>{format!("{} joined", props.user)}</p>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct LeftProps {
    pub user: String
}

#[function_component(LeftMessage)]
pub fn left_message(props: &LeftProps) -> Html {
    todo!()
}

#[derive(PartialEq, Properties)]
pub struct SentMessageProps {
    pub user: String,
    pub message: String,
}

#[function_component(SentMessage)]
pub fn sent_message(props: &SentMessageProps) -> Html {
    todo!()
}