use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;
use state::State;
use middleware::Middleware;

type ArcRwLockMiddleware<T> = Arc<RwLock<Middleware<T> + Send + Sync + 'static>>;

#[derive(Clone)]
pub struct Store<T> where T: State + Clone {
    state: Arc<RwLock<T>>,
    middlewares: Vec<ArcRwLockMiddleware<T>>,
}

impl <T> Store<T> where
    T: State + Send + Sync + Clone + 'static,
    T::Action: Send {
    pub fn new(state: T) -> Self {
        Store {
            state: Arc::new(RwLock::new(state)),
            middlewares: Vec::new(),
        }
    }

    pub fn add_middleware<M>(&mut self, middleware: M) where
        M: Middleware<T> + Send + Sync + 'static {
        self.middlewares.push(Arc::new(RwLock::new(middleware)));
    }

    pub fn state(&self) -> RwLockReadGuard<T> {
        self.state.read().unwrap()
    }

    pub fn dispatch(&mut self, action: T::Action) {
        self.dispatch_middleware(0, action);
    }

    fn dispatch_middleware(&mut self, index: usize, action: T::Action) {
        if index < self.middlewares.len() {
            if let Ok(mut middleware) = self.middlewares[index].try_write() {
                let mut store = self.clone();
                let mut next = |action: T::Action| {
                    store.dispatch_middleware(index + 1, action);
                };
                let mut store = self.clone();
                middleware.dispatch(&mut store, &mut next, action);
            } else {
                let mut store = self.clone();
                thread::spawn(move || {
                    store.dispatch_middleware(index, action);
                });
            }
        } else {
            self.state.write().unwrap().reduce(action);
        }
    }
}

