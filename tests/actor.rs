extern crate crux;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

use crux::{Actor, ActorStore, spawn_actor};

struct ExampleActor {
    count: i32,
    sender: Arc<Mutex<Sender<i32>>>,
}

impl Actor for ExampleActor {
    type Action = i32;
    fn receive(&mut self, action: i32) {
        self.count += action;
        self.sender.lock().unwrap().send(self.count).unwrap();
    }
}

#[test]
fn example() {
    let (sender, receiver) = channel();
    let actor = ExampleActor {
        count: 0,
        sender: Arc::new(Mutex::new(sender)),
    };
    let mut store = spawn_actor(actor);
    store.dispatch(1);
    assert_eq!(receiver.recv().unwrap(), 1);
    store.dispatch(2);
    assert_eq!(receiver.recv().unwrap(), 3);
    store.dispatch(3);
    assert_eq!(receiver.recv().unwrap(), 6);
}

struct RelayActor {
    count: i32,
    actor: ActorStore<ExampleActor>,
    sender: Arc<Mutex<Sender<i32>>>,
}

impl Actor for RelayActor {
    type Action = i32;
    fn receive(&mut self, action: i32) {
        self.count += action;
        self.actor.dispatch(action);
        self.sender.lock().unwrap().send(self.count).unwrap();
    }
}

#[test]
fn relay() {
    let (sender, receiver) = channel();
    let (relay_sender, relay_receiver) = channel();
    let actor = ExampleActor {
        count: 0,
        sender: Arc::new(Mutex::new(sender)),
    };
    let actor = spawn_actor(actor);
    let relay_actor = RelayActor {
        count: 0,
        actor: actor,
        sender: Arc::new(Mutex::new(relay_sender)),
    };
    let mut relay = spawn_actor(relay_actor);
    relay.dispatch(1);
    assert_eq!(receiver.recv().unwrap(), 1);
    assert_eq!(relay_receiver.recv().unwrap(), 1);
    relay.dispatch(2);
    assert_eq!(receiver.recv().unwrap(), 3);
    assert_eq!(relay_receiver.recv().unwrap(), 3);
    relay.dispatch(3);
    assert_eq!(receiver.recv().unwrap(), 6);
    assert_eq!(relay_receiver.recv().unwrap(), 6);
}
