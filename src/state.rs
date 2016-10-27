pub trait State {
    type Action;
    fn reduce(&mut self, action: Self::Action);
}
