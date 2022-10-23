use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::Mutex;

use crate::package::metadata::MetadataVersion;

#[derive(Debug, Clone)]
pub struct State {
    pub shared: Arc<Mutex<SharedState>>,
}

#[derive(Debug)]
pub struct SharedState {
    pub dependencies: HashMap<String, MetadataVersion>,
    pub dependencies_in_progress: HashSet<String>,
    pub progress: ProgressBar,
}

impl State {
    pub fn new() -> Self {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::with_template(
                "{spnner:.green} [{elapsed_precise}] [{bar}] {pos}/{len} >> {msg}",
            )
            .unwrap(),
        );

        let shared = Arc::new(Mutex::new(SharedState {
            dependencies: HashMap::new(),
            dependencies_in_progress: HashSet::new(),
            progress: pb,
        }));

        State { shared }
    }
}
