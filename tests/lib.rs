extern crate crux;

use std::time::Duration;
use std::thread;
use crux::{State, Store, Middleware};

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
        if let TestAction::Add(x) = action {
            self.number += x;
        }
    }
}

struct BonusTimeMiddleware {
    pub counter: i32,
}

impl Middleware<TestState> for BonusTimeMiddleware {
    fn dispatch(&mut self, store: &mut Store<TestState>, next: &mut FnMut(TestAction), action: TestAction) {
        if let TestAction::BonusTime = action {
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
        }
        next(action);
    }
}

struct ReplaceMiddleware;

impl Middleware<TestState> for ReplaceMiddleware {
    fn dispatch(&mut self, store: &mut Store<TestState>, next: &mut FnMut(TestAction), action: TestAction) {
        match action {
            TestAction::Increment => {
                store.dispatch(TestAction::Add(1));
            },
            TestAction::Decrement => {
                store.dispatch(TestAction::Add(-1));
            },
            _ => {
                next(action);
            },
        }
    }
}

#[test]
fn example_sync() {
    let state = TestState {
        number: 0,
    };
    let mut store = Store::new(state);

    let bonus_time_middleware = BonusTimeMiddleware {
        counter: 0,
    };
    let replace_middleware = ReplaceMiddleware;

    store.add_middleware(bonus_time_middleware);
    store.add_middleware(replace_middleware);

    assert_eq!(store.state().number, 0);

    store.dispatch_sync(TestAction::Add(2));
    assert_eq!(store.state().number, 2);

    store.dispatch_sync(TestAction::Increment);
    assert_eq!(store.state().number, 3);

    // start BonusTime 1
    store.dispatch(TestAction::BonusTime);

    store.dispatch_sync(TestAction::Add(3));
    assert_eq!(store.state().number, 6);

    store.dispatch_sync(TestAction::Decrement);
    assert_eq!(store.state().number, 5);

    // finish BonusTime 1
    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 5 + (5 - 3) * 1); // 7

    // start BonusTime 2
    store.dispatch(TestAction::BonusTime);

    store.dispatch_sync(TestAction::Increment);
    assert_eq!(store.state().number, 8);

    store.dispatch_sync(TestAction::Add(-4));
    assert_eq!(store.state().number, 4);

    // finish BonusTime 2
    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 4 + (4 - 7) * 2); // -2
}

#[test]
fn example_async() {
    let state = TestState {
        number: 0,
    };
    let mut store = Store::new(state);

    let replace_middleware = ReplaceMiddleware;

    store.add_middleware(replace_middleware);

    assert_eq!(store.state().number, 0);

    store.dispatch(TestAction::Increment);
    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 1);

    store.dispatch(TestAction::Decrement);
    thread::sleep(Duration::from_millis(25));
    assert_eq!(store.state().number, 0);
}
