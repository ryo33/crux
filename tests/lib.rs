extern crate rstate;
use rstate::state::State;
use rstate::store::Store;
use rstate::middleware::Middleware;

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

struct TestMiddleware {
    pub counter: i32,
}

impl<T> Middleware<T> for TestMiddleware where T: State {
    fn dispatch<N>(&mut self, next: N, action: T::Action) where N: Fn(T::Action) {
        self.counter += 1;
        println!("{}", self.counter);
        next(action);
    }
}

#[test]
fn store() {
    let state = TestState {
        number: 0,
    };
    let middleware = TestMiddleware {
        counter: 0,
    };
    let mut store = Store::new(state, middleware);
    {
        let current_state = store.state();
        assert_eq!(current_state.number, 0);
    }

    store.dispatch(TestAction::Increment);
    assert_eq!(store.state().number, 1);

    store.dispatch(TestAction::Add(2));
    assert_eq!(store.state().number, 3);

    store.dispatch(TestAction::Decrement);
    assert_eq!(store.state().number, 2);
}
