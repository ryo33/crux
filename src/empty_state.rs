use std::marker::PhantomData;
use state::State;

#[derive(Debug, Clone)]
pub struct EmptyState<A> {
   action_type: PhantomData<A>,
}

impl<A> EmptyState<A> {
    pub fn new() -> EmptyState<A> {
        EmptyState {
            action_type: PhantomData,
        }
    }
}

impl<A> State for EmptyState<A> {
    type Action = A;
    fn reduce(&mut self, _action: A) {}
}
