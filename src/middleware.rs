use state::State;

pub trait Middleware<T> where T: State {
    fn dispatch(&mut self, next: &Fn(T::Action), action: T::Action);
}
