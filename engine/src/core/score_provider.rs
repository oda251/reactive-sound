use std::time::Instant;

use crate::core::event::InputEvent;
use crate::core::scheduler::Score;

/// Boundary trait between Input and Output layers.
/// Receives raw input events, produces a Score for playback.
/// Implementations bundle their own recording and interpretation logic.
pub trait ScoreProvider {
    fn on_event(&mut self, event: &InputEvent, now: Instant);
    fn score(&self, now: Instant) -> Score;
}
