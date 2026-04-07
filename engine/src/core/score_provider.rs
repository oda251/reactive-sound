use std::time::Instant;

use crate::core::event::InputEvent;
use crate::core::scheduler::PatternSlot;

/// Legacy boundary trait. Prefer using ImmediateEffect / AccumulativeEffect instead.
pub trait ScoreProvider {
    fn on_event(&mut self, event: &InputEvent, now: Instant);
    fn score(&self, now: Instant) -> PatternSlot;
}
