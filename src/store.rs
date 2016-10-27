use state::State;

pub struct Store<T> where T: State {
    pub state: T,
}

impl<T> Store<T> where T: State {
    pub fn new(state: T) -> Self {
        Store {
            state: state,
        }
    }

    pub fn dispatch(&mut self, action: T::Action) {
        self.state.reduce(action);
    }
}

