use indicatif::ProgressBar;

pub trait ProgressHandler: Send + Sync {
    fn progress_set_length(&self, length: u64);
    fn progress_increment_length(&self, value: u64);
    fn progress_set(&self, value: u64);
    fn progress_increment(&self, value: u64);
    fn progress_done(&self);
    fn progress_done_with_message(&self, msg: String);
}

impl ProgressHandler for ProgressBar {
    fn progress_set_length(&self, length: u64) {
        self.set_length(length);
    }

    fn progress_increment_length(&self, value: u64) {
        self.inc_length(value);
    }

    fn progress_set(&self, value: u64) {
        self.set_position(value);
    }

    fn progress_increment(&self, value: u64) {
        self.inc(value);
    }

    fn progress_done(&self) {
        self.finish_and_clear();
    }

    fn progress_done_with_message(&self, msg: String) {
        self.finish_with_message(msg);
    }
}
