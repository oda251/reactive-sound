use std::time::Instant;

use crate::core::event::InputEvent;
use crate::core::scheduler::Score;

/// Boundary trait between Input and Output layers.
///
/// Data flow:
///   [Input Adapters] -- InputEvent --> on_event() --> [internal state] --> score() -- Score --> [Engine]
///
/// Implementations bundle recording (storing raw input) and interpretation
/// (converting raw input to musical events) as a single swappable unit.
/// The app calls on_event() for each input, then periodically calls score()
/// to get the current Score and sends it to the Engine.
pub trait ScoreProvider {
    fn on_event(&mut self, event: &InputEvent, now: Instant);
    fn score(&self, now: Instant) -> Score;
}
