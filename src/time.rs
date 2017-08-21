use std::time::{
    Duration,
    Instant,
};


pub struct Time {
    delta_time: Duration,
    last_frame_time: Instant,
}

impl Time {
    // FIXME: This shouldn't be accessible outside the crate.
    pub fn new() -> Self {
        Time {
            delta_time: Duration::from_secs(0),
            last_frame_time: Instant::now(),
        }
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn update(&mut self) {
        let frame_time = Instant::now();
        self.delta_time = frame_time - self.last_frame_time;
        self.last_frame_time = frame_time;
    }

    pub fn delta_time(&self) -> f64 {
        Self::duration_as_f64(self.delta_time)
    }

    pub fn duration_as_f64(duration: Duration) -> f64 {
        duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1_000_000_000.0)
    }
}
