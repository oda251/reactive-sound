use reactive_bgm_engine::InputEvent;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const DEFAULT_WINDOW: Duration = Duration::from_secs(20);

pub struct Recorder {
    events: VecDeque<Instant>,
    window: Duration,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            window: DEFAULT_WINDOW,
        }
    }

    pub fn record(&mut self, event: &InputEvent, now: Instant) {
        match event {
            InputEvent::KeyPress { timestamp } => {
                self.events.push_back(*timestamp);
            }
        }
        self.prune(now);
    }

    /// Returns all recorded timestamps within the window, oldest first.
    pub fn timestamps(&self) -> &VecDeque<Instant> {
        &self.events
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    fn prune(&mut self, now: Instant) {
        let cutoff = now.checked_sub(self.window).unwrap_or(now);
        while let Some(&front) = self.events.front() {
            if front < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }
}
