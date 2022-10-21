use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::package::metadata::MetadataVersion;

pub struct StateGuard {
    state: State,
}

#[derive(Debug, Clone)]
pub struct State {
    pub shared: Arc<Mutex<SharedState>>,
}

#[derive(Debug)]
pub struct SharedState {
    pub dependencies: HashMap<String, MetadataVersion>,
    pub dependencies_in_progress: HashSet<String>,
}

impl StateGuard {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    pub fn state(&self) -> State {
        self.state.clone()
    }
}

impl State {
    pub fn new() -> Self {
        let shared = Arc::new(Mutex::new(SharedState {
            dependencies: HashMap::new(),
            dependencies_in_progress: HashSet::new(),
        }));
        State { shared }
    }
}
