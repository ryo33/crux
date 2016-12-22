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
    dispatch_sender: Arc<Mutex<SyncSender<(T::Action, bool)>>>, // (action, is_sync)
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
            dispatch_sender: Arc::new(Mutex::new(dispatch_sender)),
            dispatch_receiver: Arc::new(Mutex::new(dispatch_receiver)),
        };
        let mut store_mut = store.clone();
        thread::spawn(move || {
            let mut sync = false;
            loop {
                let (action, s) = receiver.recv().unwrap();
                if s == true {
                    sync = true;
                }
                store_mut.dispatch_(action);
                let mut actions = store_mut.processing_actions.lock().unwrap();
                *actions -= 1;
                if sync && *actions == 0 {
                    sender.send(()).unwrap();
                    sync = false;
                }
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
        self.send_dispatch(action, false);
    }

    pub fn dispatch_sync(&mut self, action: T::Action) {
        self.send_dispatch(action, true);
        self.dispatch_receiver.lock().unwrap().recv().unwrap();
    }

    fn send_dispatch(&mut self, action: T::Action, sync: bool) {
        *self.processing_actions.lock().unwrap() += 1;
        self.dispatch_sender.lock().unwrap().send((action, sync)).unwrap();
    }

    fn dispatch_(&mut self, action: T::Action) {
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

