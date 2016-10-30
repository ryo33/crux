use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::slice::IterMut;
use state::State;
use middleware::Middleware;

type ArcRwLockMiddleware<T> = Arc<RwLock<Middleware<T> + Send + Sync + 'static>>;

#[derive(Clone)]
pub struct Store<T> where T: State + Clone {
    state: Arc<RwLock<T>>,
    middlewares: Vec<ArcRwLockMiddleware<T>>,
}

impl <T> Store<T> where T: State + Clone {
    pub fn new(state: T) -> Self {
        Store {
            state: Arc::new(RwLock::new(state)),
            middlewares: Vec::new(),
        }
    }

    pub fn add_middleware<M>(&mut self, middleware: M)
        where M: Middleware<T> + Send + Sync + 'static {
        self.middlewares.push(Arc::new(RwLock::new(middleware)));
    }

    pub fn state(&self) -> RwLockReadGuard<T> {
        self.state.read().unwrap()
    }

    pub fn dispatch(&mut self, action: T::Action) {
        let mut middlewares = self.middlewares.clone();
        let mut iter = middlewares.iter_mut();
        self.dispatch_middleware(&mut iter, action);
    }

    fn dispatch_middleware(&mut self, iter: &mut IterMut<ArcRwLockMiddleware<T>>, action: T::Action) {
        match iter.next() {
            Some(middleware) => {
                let store = self.clone();
                let mut next = |action: T::Action| {
                    self.dispatch_middleware(iter, action);
                };
                middleware.write().unwrap().dispatch(store, &mut next, action);
            },
            _ => {
                self.state.write().unwrap().reduce(action);
            }
        }
    }
}

