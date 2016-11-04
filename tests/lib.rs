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
    AssertEq(i32),
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

struct AssertMiddleware;

impl Middleware<TestState> for AssertMiddleware {
    fn dispatch(&mut self, store: &mut Store<TestState>, next: &mut FnMut(TestAction), action: TestAction) {
        if let TestAction::AssertEq(number) = action {
            assert_eq!(store.state().number, number);
        }
        next(action);
    }
}

#[test]
fn example() {
    let state = TestState {
        number: 0,
    };
    let mut store = Store::new(state);

    let bonus_time_middleware = BonusTimeMiddleware {
        counter: 0,
    };
    let replace_middleware = ReplaceMiddleware;
    let assert_middleware = AssertMiddleware;

    store.add_middleware(bonus_time_middleware);
    store.add_middleware(replace_middleware);
    store.add_middleware(assert_middleware);

    store.dispatch(TestAction::AssertEq(0));

    store.dispatch(TestAction::Increment);
    thread::sleep(Duration::from_millis(1));
    store.dispatch(TestAction::AssertEq(1));

    store.dispatch(TestAction::Add(2));
    store.dispatch(TestAction::AssertEq(3));

    // start BonusTime 1
    store.dispatch(TestAction::BonusTime);

    store.dispatch(TestAction::Add(3));
    store.dispatch(TestAction::AssertEq(6));

    store.dispatch(TestAction::Decrement);
    thread::sleep(Duration::from_millis(1));
    store.dispatch(TestAction::AssertEq(5));

    // finish BonusTime 1
    thread::sleep(Duration::from_millis(25));
    store.dispatch(TestAction::AssertEq(5 + (5 - 3) * 1)); // 7

    // start BonusTime 2
    store.dispatch(TestAction::BonusTime);

    store.dispatch(TestAction::Add(-4));
    store.dispatch(TestAction::AssertEq(3));

    store.dispatch(TestAction::Increment);
    thread::sleep(Duration::from_millis(1));
    store.dispatch(TestAction::AssertEq(4));

    // finish BonusTime 2
    thread::sleep(Duration::from_millis(25));
    store.dispatch(TestAction::AssertEq(4 + (4 - 7) * 2)); // -2

    assert_eq!(store.state().number, -2);
}
