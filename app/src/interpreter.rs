use reactive_bgm_engine::{NoteEvent, Score};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub const LOOP_DURATION_SECS: f32 = 20.0;
pub const MEASURES: usize = 10;

const NOTE_DURATION: f32 = 0.08;
const HIT_NOTES: [u8; 4] = [60, 67, 72, 63];

pub trait Interpreter {
    fn interpret(&self, timestamps: &VecDeque<Instant>, snapshot_time: Instant) -> Score;
}

pub struct RawRhythmInterpreter;

impl Interpreter for RawRhythmInterpreter {
    fn interpret(&self, timestamps: &VecDeque<Instant>, snapshot_time: Instant) -> Score {
        let loop_dur = Duration::from_secs_f32(LOOP_DURATION_SECS);
        let window_start = snapshot_time.checked_sub(loop_dur).unwrap_or(snapshot_time);

        let mut hit_idx = 0;
        let events = timestamps
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
