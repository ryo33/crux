extern crate crux;
use std::time::Duration;
use std::thread;
use crux::State;
use crux::Store;
use crux::Middleware;

#[derive(Debug, Copy, Clone)]
enum TestAction {
    Add(i32),
    Increment,
    Decrement,
    BonusTime, // 20ms Bonus Time
}

#[derive(Debug, Clone)]
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
            _ => {},
        }
    }
}

struct TestMiddleware {
    pub counter: i32,
}

impl Middleware<TestState> for TestMiddleware {
    fn dispatch(&mut self, store: Store<TestState>, next: &mut FnMut(TestAction), action: TestAction) {
        match action {
            TestAction::BonusTime => {
                self.counter += 1;

                let counter = self.counter;
                let mut store_mut = store.clone();

                thread::spawn(move || {
                    let number = store_mut.state().number;
                    thread::sleep(Duration::from_millis(20));
                    let next_number = store_mut.state().number;
                    let bonus = (next_number - number) * counter;
                    store_mut.dispatch(TestAction::Add(bonus));
                });
                thread::sleep(Duration::from_millis(1));
                next(action);
            },
            _ => {
                next(action);
            },
        }
    }
}

struct Logger;
impl Middleware<TestState> for Logger {
    fn dispatch(&mut self, _store: Store<TestState>, next: &mut FnMut(TestAction), action: TestAction) {
        // println!("previous state: {:?}", store.state());
        next(action);
        println!("action: {:?}", action);
        // println!("next state: {:?}", store.state());
    }
}

#[test]
fn store() {
    let state = TestState {
        number: 0,
    };
    let mut store = Store::new(state);

    let logger = Logger;
    let middleware = TestMiddleware {
        counter: 0,
    };

    store.add_middleware(middleware);
    store.add_middleware(logger);

    assert_eq!(store.state().number, 0);

    store.dispatch(TestAction::Increment);
    assert_eq!(store.state().number, 1);

    store.dispatch(TestAction::Add(2));
    assert_eq!(store.state().number, 3);

    // start BonusTime
    store.dispatch(TestAction::BonusTime);

    store.dispatch(TestAction::Add(3));
    assert_eq!(store.state().number, 6);

    store.dispatch(TestAction::Decrement);
    assert_eq!(store.state().number, 5);

    // finish BonusTime
    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 5 + (5 - 3) * 1); // 7

    // start BonusTime
    store.dispatch(TestAction::BonusTime);

    store.dispatch(TestAction::Add(-4));
    assert_eq!(store.state().number, 3);

    store.dispatch(TestAction::Increment);
    assert_eq!(store.state().number, 4);

    // finish BonusTime
    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 4 + (4 - 7) * 2); // -1
}
