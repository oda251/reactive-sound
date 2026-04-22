use rand::rngs::ThreadRng;
use rand::Rng;
use reactive_bgm_engine::{
    AccumulativeEffect, ImmediateAction, ImmediateEffect, InputEffect, InputEvent, NoteEvent,
    PatternSlot, VoiceType, TICKS_PER_BEAT,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Loop parameters — SSOT for timing.
#[derive(Clone, Debug)]
pub struct LoopConfig {
    pub measures: usize,
    pub beats_per_measure: u32,
    #[allow(dead_code)]
    pub bpm: f64,
    loop_duration_secs: f32,
    measure_duration_secs: f32,
}

impl LoopConfig {
    pub fn new(measures: usize, beats_per_measure: u32, bpm: f64) -> Self {
        let total_beats = measures as f64 * beats_per_measure as f64;
        let loop_secs = total_beats * 60.0 / bpm;
        let measure_secs = beats_per_measure as f64 * 60.0 / bpm;
        Self {
            measures,
            beats_per_measure,
            bpm,
            loop_duration_secs: loop_secs as f32,
            measure_duration_secs: measure_secs as f32,
        }
    }

    pub fn total_ticks(&self) -> u32 {
        TICKS_PER_BEAT * self.measures as u32 * self.beats_per_measure
    }

    pub fn loop_duration(&self) -> Duration {
        Duration::from_secs_f32(self.loop_duration_secs)
    }

    pub fn loop_duration_secs(&self) -> f32 {
        self.loop_duration_secs
    }

    pub fn measure_duration_secs(&self) -> f32 {
        self.measure_duration_secs
    }
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self::new(5, 4, 120.0)
    }
}

// --- PatternStrategy ---

/// Swappable: converts note positions (0.0..1.0) into NoteEvents.
pub trait PatternStrategy {
    fn generate(&mut self, positions: &[f32], total_ticks: u32) -> Vec<NoteEvent>;
}

/// A minor pentatonic scale, random note per hit.
const AM_PENTA: [u8; 15] = [
    57, 60, 62, 64, 67,
    69, 72, 74, 76, 79,
    81, 84, 86, 88, 91,
];

pub struct PentatonicRandom {
    rng: ThreadRng,
}

impl PentatonicRandom {
    pub fn new() -> Self {
        Self { rng: rand::rng() }
    }
}

const SIXTEENTH_NOTE: u32 = TICKS_PER_BEAT / 4;

impl PatternStrategy for PentatonicRandom {
    fn generate(&mut self, positions: &[f32], total_ticks: u32) -> Vec<NoteEvent> {
        positions
            .iter()
            .map(|&pos| {
                let note = AM_PENTA[self.rng.random_range(0..AM_PENTA.len())];
                NoteEvent {
                    tick: (pos * total_ticks as f32) as u32,
                    note,
                    duration_ticks: SIXTEENTH_NOTE,
                    gain: 0.25,
                    voice_type: VoiceType(0),
                    overrides: Vec::new(),
                }
            })
            .collect()
    }
}

// --- AccumulativeEffect ---

pub struct RhythmAccumulator {
    timestamps: VecDeque<Instant>,
    epoch: Instant,
    loop_config: LoopConfig,
    strategy: Box<dyn PatternStrategy>,
}

impl RhythmAccumulator {
    pub fn new(epoch: Instant) -> Self {
        Self::with_strategy(epoch, Box::new(PentatonicRandom::new()))
    }

    pub fn with_strategy(epoch: Instant, strategy: Box<dyn PatternStrategy>) -> Self {
        Self {
            timestamps: VecDeque::new(),
            epoch,
            loop_config: LoopConfig::default(),
            strategy,
        }
    }

    pub fn measures(&self) -> usize {
        self.loop_config.measures
    }

    pub fn event_count(&self) -> usize {
        self.timestamps.len()
    }

    fn loop_position(&self, ts: Instant) -> f32 {
        let offset = ts.duration_since(self.epoch).as_secs_f32();
        let window_secs = self.loop_config.loop_duration_secs();
        (offset % window_secs) / window_secs
    }

    pub fn live_positions(&self) -> Vec<f32> {
        self.timestamps.iter().map(|&ts| self.loop_position(ts)).collect()
    }

    pub fn input_cursor(&self, now: Instant) -> f32 {
        let measure_secs = self.loop_config.measure_duration_secs();
        let elapsed = now.duration_since(self.epoch).as_secs_f32();
        (elapsed / measure_secs).fract()
    }

    pub fn current_measure_notes(&self, now: Instant) -> Vec<f32> {
        let measure_secs = self.loop_config.measure_duration_secs();
        let elapsed = now.duration_since(self.epoch).as_secs_f32();
        let measure_start_elapsed = (elapsed / measure_secs).floor() * measure_secs;
        let measure_start = self.epoch + Duration::from_secs_f32(measure_start_elapsed);
        let measure_end = measure_start + Duration::from_secs_f32(measure_secs);

        let mut notes = Vec::new();
        for &ts in self.timestamps.iter().rev() {
            if ts < measure_start {
                break;
            }
            if ts < measure_end {
                let pos = ts.duration_since(measure_start).as_secs_f32() / measure_secs;
                notes.push(pos.clamp(0.0, 1.0));
            }
        }
        notes.reverse();
        notes
    }

    fn prune(&mut self, now: Instant) {
        let cutoff = now
            .checked_sub(self.loop_config.loop_duration())
            .unwrap_or(now);
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
    fn on_event(&mut self, event: &InputEvent, _now: Instant) {
        if let InputEvent::KeyPress { timestamp } = event {
            self.timestamps.push_back(*timestamp);
        }
    }
}

impl AccumulativeEffect for RhythmAccumulator {
    fn tick(&mut self, now: Instant) {
        self.prune(now);
    }

    #[allow(unused_variables)]
    fn score(&mut self, now: Instant) -> PatternSlot {
        let total_ticks = self.loop_config.total_ticks();
        let positions = self.live_positions();
        let events = self.strategy.generate(&positions, total_ticks);

        PatternSlot {
            notes: events,
            params: Vec::new(),
            total_ticks,
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
            InputEvent::KeyPress { .. } | InputEvent::MouseClick { .. } => {
                self.pending.push(ImmediateAction::Click { gain: 0.25 });
            }
        }
    }
}

impl ImmediateEffect for KeyClickEffect {
    fn drain_actions(&mut self) -> Vec<ImmediateAction> {
        std::mem::take(&mut self.pending)
    }
}
