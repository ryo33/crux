extern crate rstate;
use std::time::Duration;
use std::thread;
use rstate::State;
use rstate::Store;
use rstate::Middleware;

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

impl Middleware<TestState> for TestMiddleware {
    fn dispatch(&mut self, store: Store<TestState, Self>, next: &Fn(TestAction), action: TestAction) {
        let mut store_mut = store.clone();
        let counter = self.counter;
        match action {
            TestAction::Add(_) => (),
            _ => {
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(5));
                    let number = store_mut.state().number;
                    thread::sleep(Duration::from_millis(15));
                    let next_number = store_mut.state().number;
                    store_mut.dispatch(TestAction::Add(next_number - number + counter));
                });
            }
        }
        self.counter += 1;
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
    // counter = 0
    assert_eq!(store.state().number, 1);

    thread::sleep(Duration::from_millis(10));

    store.dispatch(TestAction::Add(2));
    // counter = 1
    assert_eq!(store.state().number, 3);

    thread::sleep(Duration::from_millis(15));
    // counter = 2
    assert_eq!(store.state().number, 3 + (3 - 1) + 0); // 5
    // counter = 3
    thread::sleep(Duration::from_millis(15));
    // counter = 4
    assert_eq!(store.state().number, 5);

    store.dispatch(TestAction::Decrement);
    // counter = 5
    assert_eq!(store.state().number, 4);
    store.dispatch(TestAction::Add(-2));
    // counter = 6
    assert_eq!(store.state().number, 2);

    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 2 + (2 - 4) + 5);
}
