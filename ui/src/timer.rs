use std::time::{Duration, Instant};

pub(crate) struct Timer {
    start: Instant,
    entries: Vec<(String, Instant)>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            entries: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.entries.truncate(0);
    }

    pub fn stage_end<S: ToString>(&mut self, name: S) {
        self.entries.push((name.to_string(), Instant::now()));
    }

    pub fn get_durations(&self) -> Vec<(&str, Duration)> {
        self.entries
            .iter()
            .scan(self.start, |stage_start, (stage_name, stage_end)| {
                let rv = (stage_name.as_str(), stage_end.duration_since(*stage_start));
                *stage_start = *stage_end;
                Some(rv)
            })
            .collect()
    }
}
