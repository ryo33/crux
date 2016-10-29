use state::State;

pub trait Middleware<T> where T: State {
    fn dispatch<N>(&mut self, next: N, action: T::Action) where N: Fn(T::Action);
}
