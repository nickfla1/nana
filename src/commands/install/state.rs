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

    pub async fn progress_reset(&self, msg: &'static str) {
        let lock = self.shared.lock().await;
        lock.progress.reset();
        lock.progress.set_message(msg);
    }

    pub async fn progress_set_length(&self, len: u64) {
        self.shared.lock().await.progress.set_length(len);
    }

    pub async fn progress_increment_length(&self, amount: u64) {
        self.shared.lock().await.progress.inc_length(amount);
    }

    pub async fn progress_increment(&self, amount: u64) {
        self.shared.lock().await.progress.inc(amount);
    }

    pub async fn progress_finish(&self) {
        self.shared.lock().await.progress.finish_and_clear();
    }
}
