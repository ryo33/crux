use std::sync::{Arc, RwLock, Mutex, RwLockReadGuard};
use std::sync::mpsc::{channel, sync_channel, SyncSender, Receiver};
use std::thread;
use std::slice::IterMut;
use state::State;
use middleware::Middleware;

const CHANNEL_BOUND: usize = 32;

type ArcMutexMiddleware<T> = Arc<Mutex<Middleware<T> + Send + Sync + 'static>>;

#[derive(Clone)]
pub struct Store<T> where T: State {
    state: Arc<RwLock<T>>,
    middlewares: Arc<RwLock<Vec<ArcMutexMiddleware<T>>>>,
    processing_actions: Arc<Mutex<i32>>,
    dispatch_sender: SyncSender<T::Action>,
    dispatch_receiver: Arc<Mutex<Receiver<()>>>,
}

impl<T> Store<T> where
    T: State + Send + Sync + Clone + 'static,
    T::Action: Send + Clone {
    pub fn new(state: T) -> Self {
        let (dispatch_sender, receiver) = sync_channel(CHANNEL_BOUND);
        let (sender, dispatch_receiver) = channel();
        let store = Store {
            state: Arc::new(RwLock::new(state)),
            middlewares: Arc::new(RwLock::new(Vec::new())),
            processing_actions: Arc::new(Mutex::new(0)),
            dispatch_sender: dispatch_sender,
            dispatch_receiver: Arc::new(Mutex::new(dispatch_receiver)),
        };
        let mut store_mut = store.clone();
        thread::spawn(move || {
            loop {
                let action = receiver.recv().unwrap();
                store_mut._dispatch(action);
                sender.send(()).unwrap();
            }
        });
        store
    }

    pub fn add_middleware<M>(&mut self, middleware: M) where
        M: Middleware<T> + Send + Sync + 'static {
        self.middlewares.write().unwrap().push(Arc::new(Mutex::new(middleware)));
    }

    pub fn state(&self) -> RwLockReadGuard<T> {
        self.state.read().unwrap()
    }

    pub fn dispatch(&mut self, action: T::Action) {
        let from_middleware = *self.processing_actions.lock().unwrap() != 0;
        *self.processing_actions.lock().unwrap() += 1;
        self.dispatch_sender.send(action).unwrap();
        if ! from_middleware {
            while *self.processing_actions.lock().unwrap() != 0 {
                self.dispatch_receiver.lock().unwrap().recv().unwrap();
                *self.processing_actions.lock().unwrap() -= 1;
            }
        }
    }

    pub fn _dispatch(&mut self, action: T::Action) {
        let mut middlewares = self.middlewares.write().unwrap();
        let mut iter = middlewares.iter_mut();
        let mut store = self.clone();
        store.dispatch_middleware(&mut iter, action);
    }

    fn dispatch_middleware(&mut self, iter: &mut IterMut<ArcMutexMiddleware<T>>, action: T::Action) {
        match iter.next() {
            Some(middleware) => {
                let mut store = self.clone();
                let mut next = |action: T::Action| {
                    store.dispatch_middleware(iter, action);
                };
                let mut store = self.clone();
                middleware.lock().unwrap().dispatch(&mut store, &mut next, action);
            },
            _ => {
                self.state.write().unwrap().reduce(action);
            }
        }
    }
}

