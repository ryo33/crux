use std::sync::{Arc, Mutex, MutexGuard};
use state::State;
use middleware::Middleware;

pub struct Store<T, M> {
    state: Arc<Mutex<T>>,
    middleware: Arc<Mutex<M>>,
}

impl <T, M> Store<T, M> where T: State, M: Middleware<T> {
    pub fn new(state: T, middleware: M) -> Self {
        Store {
            state: Arc::new(Mutex::new(state)),
            middleware: Arc::new(Mutex::new(middleware)),
        }
    }

    pub fn clone(&self) -> Self {
        Store {
            state: self.state.clone(),
            middleware: self.middleware.clone(),
        }
    }

    pub fn dispatch(&mut self, action: T::Action) {
        let store = self.clone();
        let next = |action: T::Action| {
            self.state.lock().unwrap().reduce(action);
        };
        self.middleware.lock().unwrap().dispatch(store, &next, action);
    }

    pub fn state(&self) -> MutexGuard<T> {
        self.state.lock().unwrap()
    }
}
