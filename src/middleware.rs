use state::State;
use store::Store;

pub trait Middleware<T> where T: State, Self: Sized {
    fn dispatch(&mut self, store: Store<T, Self>, next: &Fn(T::Action), action: T::Action);
}
