use reactive_bgm_engine::{InputEvent, NoteEvent, Score, ScoreProvider};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const LOOP_DURATION_SECS: f32 = 20.0;
const WINDOW: Duration = Duration::from_secs(20);
const NOTE_DURATION: f32 = 0.08;
const HIT_NOTES: [u8; 4] = [60, 67, 72, 63];

pub const MEASURES: usize = 10;

pub struct RawRhythmProvider {
    timestamps: VecDeque<Instant>,
}

impl RawRhythmProvider {
    pub fn new() -> Self {
        Self {
            timestamps: VecDeque::new(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.timestamps.len()
    }

    /// Committed note positions for GUI display (0.0..1.0 in loop).
    pub fn note_positions(&self, now: Instant) -> Vec<f32> {
        let window_start = now.checked_sub(WINDOW).unwrap_or(now);
        self.timestamps
            .iter()
            .filter_map(|&ts| {
                let offset = ts.checked_duration_since(window_start)?;
                let pos = offset.as_secs_f32() / LOOP_DURATION_SECS;
                if (0.0..1.0).contains(&pos) {
                    Some(pos)
                } else {
                    None
                }
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

impl ScoreProvider for RawRhythmProvider {
    fn on_event(&mut self, event: &InputEvent, now: Instant) {
        match event {
            InputEvent::KeyPress { timestamp } => {
                self.timestamps.push_back(*timestamp);
            }
        }
        self.prune(now);
    }

    fn score(&self, now: Instant) -> Score {
        let window_start = now.checked_sub(WINDOW).unwrap_or(now);

        let mut hit_idx = 0;
        let events = self.timestamps
            .iter()
            .filter_map(|&ts| {
                let offset = ts.checked_duration_since(window_start)?;
                let pos = offset.as_secs_f32() / LOOP_DURATION_SECS;
                if (0.0..1.0).contains(&pos) {
                    let note = HIT_NOTES[hit_idx % HIT_NOTES.len()];
                    hit_idx += 1;
                    Some(NoteEvent {
                        position: pos,
                        note,
                        duration: NOTE_DURATION,
                    })
                } else {
                    None
                }
            })
            .collect();

        Score {
            events,
            loop_duration_secs: LOOP_DURATION_SECS,
        }
    }
}
