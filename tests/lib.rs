extern crate rstate;
use rstate::state::State;
use rstate::store::Store;

enum TestAction {
    Increment,
    Decrement,
    Add(i32),
}

struct TestState {
    pub number: i32,
}

impl State for TestState {
    type Action = TestAction;

    fn reduce(&mut self, action: TestAction) {
        match action {
            TestAction::Add(x) => self.number += x,
            TestAction::Increment => self.number += 1,
            TestAction::Decrement => self.number -= 1,
        }
    }
}

#[test]
fn test() {
    let state = TestState {
        number: 0
    };
    let mut store = Store::new(state);
    assert_eq!(store.state.number, 0);

    store.dispatch(TestAction::Increment);
    assert_eq!(store.state.number, 1);

    store.dispatch(TestAction::Add(2));
    assert_eq!(store.state.number, 3);

    store.dispatch(TestAction::Decrement);
    assert_eq!(store.state.number, 2);
}
