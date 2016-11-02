use state::State;
use store::Store;

pub trait Middleware<T> where T: State + Clone {
    fn dispatch(&mut self, store: &mut Store<T>, next: &mut FnMut(T::Action), action: T::Action);
}
