use std::time::Instant;

use crate::core::event::InputEvent;
use crate::core::scheduler::PatternSlot;

#[derive(Debug, Clone)]
pub enum ImmediateAction {
    NoteOn { note: u8, gain: f32 },
    NoteOff { note: u8 },
    NoteOnOff {
        note: u8,
        gain: f32,
        duration_samples: u32,
    },
    Click { gain: f32 },
    SetParam(i32, f32),
}

/// Base trait: receives input events.
pub trait InputEffect {
    fn on_event(&mut self, event: &InputEvent, now: Instant);
}

/// Produces immediate actions in response to input.
pub trait ImmediateEffect: InputEffect {
    fn drain_actions(&mut self) -> Vec<ImmediateAction>;
}

/// Accumulates input over time and produces a PatternSlot for looped playback.
pub trait AccumulativeEffect: InputEffect {
    fn tick(&mut self, now: Instant);
    fn score(&mut self, now: Instant) -> PatternSlot;
}
