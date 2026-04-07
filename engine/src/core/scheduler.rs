/// A note event in the score: position (0.0..1.0 within loop), MIDI note, duration in seconds.
#[derive(Clone, Debug)]
pub struct NoteEvent {
    pub position: f32,
    pub note: u8,
    pub duration: f32,
}

/// A loop score: a collection of note events over a fixed duration.
#[derive(Clone, Debug)]
pub struct Score {
    pub events: Vec<NoteEvent>,
    pub loop_duration_secs: f32,
}

impl Score {
    pub fn empty(loop_duration_secs: f32) -> Self {
        Self {
            events: Vec::new(),
            loop_duration_secs,
        }
    }
}

/// Tracks playback position and determines which notes should be triggered.
pub struct Scheduler {
    sample_rate: u32,
    samples_elapsed: u64,
    score: Score,
}

impl Scheduler {
    pub fn new(sample_rate: u32, score: Score) -> Self {
        Self {
            sample_rate,
            samples_elapsed: 0,
            score,
        }
    }

    pub fn set_score(&mut self, score: Score) {
        self.score = score;
        // Don't reset samples_elapsed — keep playhead position for seamless transition
    }

    /// Current playhead position as fraction of loop (0.0..1.0).
    pub fn playhead(&self) -> f32 {
        let loop_samples = (self.score.loop_duration_secs * self.sample_rate as f32) as u64;
        if loop_samples == 0 {
            return 0.0;
        }
        (self.samples_elapsed % loop_samples) as f32 / loop_samples as f32
    }

    /// Advance by `frames` samples and return note-on/off events that occur in this block.
    pub fn advance(&mut self, frames: usize) -> Vec<SchedulerEvent> {
        let loop_samples =
            (self.score.loop_duration_secs * self.sample_rate as f32) as u64;
        if loop_samples == 0 || self.score.events.is_empty() {
            self.samples_elapsed += frames as u64;
            return Vec::new();
        }

        let mut events = Vec::new();
        let block_start = self.samples_elapsed % loop_samples;
        let block_end = block_start + frames as u64;

        for note_event in &self.score.events {
            let note_sample = (note_event.position * loop_samples as f32) as u64;
            let note_off_sample = note_sample
                + (note_event.duration * self.sample_rate as f32) as u64;

            // Note-on in this block?
            if triggers_in_range(note_sample, block_start, block_end, loop_samples) {
                let offset = sample_offset(note_sample, block_start, loop_samples);
                events.push(SchedulerEvent {
                    frame_offset: offset as usize,
                    kind: EventKind::NoteOn {
                        note: note_event.note,
                    },
                });
            }

            // Note-off in this block?
            if triggers_in_range(note_off_sample % loop_samples, block_start, block_end, loop_samples) {
                let offset = sample_offset(note_off_sample % loop_samples, block_start, loop_samples);
                events.push(SchedulerEvent {
                    frame_offset: offset as usize,
                    kind: EventKind::NoteOff {
                        note: note_event.note,
                    },
                });
            }
        }

        self.samples_elapsed += frames as u64;

        events.sort_by_key(|e| e.frame_offset);
        events
    }

    pub fn score(&self) -> &Score {
        &self.score
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerEvent {
    pub frame_offset: usize,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    NoteOn { note: u8 },
    NoteOff { note: u8 },
}

fn triggers_in_range(sample: u64, block_start: u64, block_end: u64, loop_len: u64) -> bool {
    if block_end <= loop_len {
        sample >= block_start && sample < block_end
    } else {
        // Block wraps around loop boundary
        sample >= block_start || sample < (block_end % loop_len)
    }
}

fn sample_offset(sample: u64, block_start: u64, loop_len: u64) -> u64 {
    if sample >= block_start {
        sample - block_start
    } else {
        (loop_len - block_start) + sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playhead_advances() {
        let score = Score {
            events: vec![],
            loop_duration_secs: 1.0,
        };
        let mut sched = Scheduler::new(100, score);
        assert_eq!(sched.playhead(), 0.0);
        sched.advance(50);
        assert!((sched.playhead() - 0.5).abs() < 0.01);
        sched.advance(50);
        assert!(sched.playhead() < 0.01); // wrapped
    }

    #[test]
    fn note_on_off_triggered() {
        let score = Score {
            events: vec![NoteEvent {
                position: 0.5,
                note: 60,
                duration: 0.1,
            }],
            loop_duration_secs: 1.0,
        };
        let mut sched = Scheduler::new(100, score);

        // First half: no events
        let events = sched.advance(50);
        assert!(events.is_empty());

        // Second half: note-on at frame 0, note-off at frame 10
        let events = sched.advance(50);
        assert!(events.iter().any(|e| matches!(e.kind, EventKind::NoteOn { note: 60 })));
        assert!(events.iter().any(|e| matches!(e.kind, EventKind::NoteOff { note: 60 })));
    }

    fn midi_to_freq(note: u8) -> f32 {
        440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }

    #[test]
    fn midi_to_freq_a4() {
        assert!((midi_to_freq(69) - 440.0).abs() < 0.01);
    }
}
