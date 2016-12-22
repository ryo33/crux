use store::Store;
use middleware::Middleware;
use empty_state::EmptyState;

pub type ActorState<A> where A: Actor = EmptyState<A::Action>;
pub type ActorStore<A> where A: Actor = Store<ActorState<A>>;

pub trait Actor {
    type Action;
    fn receive(&mut self, action: Self::Action);
}

struct ActorMiddleware<A> {
    actor: A,
}

impl<A: Actor> Middleware<EmptyState<A::Action>> for ActorMiddleware<A> where
    A::Action: Clone {
    fn dispatch(&mut self, _store: &mut Store<EmptyState<A::Action>>, next: &mut FnMut(A::Action), action: A::Action) {
        let action_ = action.clone();
        self.actor.receive(action_);
        next(action);
    }
}

pub fn spawn_actor<A: Actor>(actor: A) -> ActorStore<A> where
    A: Send + Sync + 'static,
    A::Action: Clone + Send + Sync {
    let state: EmptyState<A::Action> = EmptyState::new();
    let mut store = Store::new(state);
    let middleware = ActorMiddleware {
        actor: actor
    };
    store.add_middleware(middleware);
    store
}
