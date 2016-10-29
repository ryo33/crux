use std::sync::{Arc, Mutex, MutexGuard};
use state::State;
use middleware::Middleware;

pub struct Store<T, M> where T: State, M: Middleware<T> {
    state: Arc<Mutex<T>>,
    middleware: Arc<Mutex<M>>,
}

impl<T, M> Store<T, M> where T: State, M: Middleware<T> {
    pub fn new(state: T, middleware: M) -> Self {
        Store {
            state: Arc::new(Mutex::new(state)),
            middleware: Arc::new(Mutex::new(middleware)),
        }
    }

    pub fn dispatch(&mut self, action: T::Action) {
        let next = |action: T::Action| {
            self.state.lock().unwrap().reduce(action);
        };
        self.middleware.lock().unwrap().dispatch(next, action);
    }

    pub fn state(&self) -> MutexGuard<T> {
        self.state.lock().unwrap()
    }
}
