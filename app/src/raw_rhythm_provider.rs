use reactive_bgm_engine::{
    AccumulativeEffect, ImmediateAction, ImmediateEffect, InputEffect, InputEvent, NoteEvent,
    PatternSlot, VoiceType, TICKS_PER_BEAT,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const LOOP_BEATS: u32 = 40; // 10 measures * 4 beats
const TOTAL_TICKS: u32 = TICKS_PER_BEAT * LOOP_BEATS;
const WINDOW: Duration = Duration::from_secs(20);
const NOTE_DURATION_TICKS: u32 = TICKS_PER_BEAT / 4; // sixteenth note
const HIT_NOTES: [u8; 4] = [60, 67, 72, 63];

pub const MEASURES: usize = 10;

// --- AccumulativeEffect ---

pub struct RhythmAccumulator {
    timestamps: VecDeque<Instant>,
}

impl RhythmAccumulator {
    pub fn new() -> Self {
        Self {
            timestamps: VecDeque::new(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.timestamps.len()
    }

    /// Note positions as 0.0..1.0 for GUI display.
    pub fn note_positions(&self, now: Instant) -> Vec<f32> {
        let window_start = now.checked_sub(WINDOW).unwrap_or(now);
        self.timestamps
            .iter()
            .filter_map(|&ts| {
                let offset = ts.checked_duration_since(window_start)?;
                let pos = offset.as_secs_f32() / WINDOW.as_secs_f32();
                if (0.0..1.0).contains(&pos) { Some(pos) } else { None }
            })
            .collect()
    }

    fn prune(&mut self, now: Instant) {
        let cutoff = now.checked_sub(WINDOW).unwrap_or(now);
        while let Some(&front) = self.timestamps.front() {
            if front < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }
}

impl InputEffect for RhythmAccumulator {
    fn on_event(&mut self, event: &InputEvent, now: Instant) {
        match event {
            InputEvent::KeyPress { timestamp } => {
                self.timestamps.push_back(*timestamp);
            }
        }
        self.prune(now);
    }
}

impl AccumulativeEffect for RhythmAccumulator {
    fn score(&self, now: Instant) -> PatternSlot {
        let window_start = now.checked_sub(WINDOW).unwrap_or(now);

        let mut hit_idx = 0;
        let events = self
            .timestamps
            .iter()
            .filter_map(|&ts| {
                let offset = ts.checked_duration_since(window_start)?;
                let pos = offset.as_secs_f32() / WINDOW.as_secs_f32();
                if (0.0..1.0).contains(&pos) {
                    let tick = (pos * TOTAL_TICKS as f32) as u32;
                    let note = HIT_NOTES[hit_idx % HIT_NOTES.len()];
                    hit_idx += 1;
                    Some(NoteEvent {
                        tick,
                        note,
                        duration_ticks: NOTE_DURATION_TICKS,
                        gain: 0.25,
                        voice_type: VoiceType(0),
                        overrides: Vec::new(),
                    })
                } else {
                    None
                }
            })
            .collect();

        PatternSlot {
            notes: events,
            params: Vec::new(),
            total_ticks: TOTAL_TICKS,
            active: true,
        }
    }
}

// --- ImmediateEffect ---

pub struct KeyClickEffect {
    pending: Vec<ImmediateAction>,
}

impl KeyClickEffect {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }
}

impl InputEffect for KeyClickEffect {
    fn on_event(&mut self, event: &InputEvent, _now: Instant) {
        match event {
            InputEvent::KeyPress { .. } => {
                // Short click: NoteOn followed by NoteOff after a brief moment.
                // The NoteOff is handled by the engine's scheduler via the
                // Faust ADSR envelope (gate off triggers release).
                self.pending.push(ImmediateAction::NoteOn {
                    note: 80,
                    gain: 0.1,
                });
                self.pending.push(ImmediateAction::NoteOff { note: 80 });
            }
        }
    }
}

impl ImmediateEffect for KeyClickEffect {
    fn drain_actions(&mut self) -> Vec<ImmediateAction> {
        std::mem::take(&mut self.pending)
    }
}
