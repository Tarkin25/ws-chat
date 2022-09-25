use yew_agent::Agent;

pub mod websocket;

pub trait StatefulAgent: Agent {
    type State;
    type Shared;
}

pub trait HandleInput<A: StatefulAgent> {
    fn handle_input(&mut self, input: A::Input, shared: &A::Shared) -> Option<A::State>;
}