use std::collections::VecDeque;

pub const TICKS_PER_BEAT: u32 = 480;

#[derive(Clone, Debug)]
pub struct NoteEvent {
    pub tick: u32,
    pub note: u8,
    pub duration_ticks: u32,
    pub gain: f32,
}

#[derive(Clone, Debug)]
pub struct PatternSlot {
    pub events: Vec<NoteEvent>,
    pub total_ticks: u32,
    pub active: bool,
}

impl PatternSlot {
    pub fn empty(total_ticks: u32) -> Self {
        Self {
            events: Vec::new(),
            total_ticks,
            active: false,
        }
    }
}

/// A one-shot scheduled note at an absolute sample time.
#[derive(Clone, Debug)]
pub struct QueuedNote {
    pub at_sample: u64,
    pub note: u8,
    pub duration_samples: u64,
    pub gain: f32,
}

#[derive(Debug, Clone)]
pub struct SchedulerEvent {
    pub frame_offset: usize,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    NoteOn { note: u8, gain: f32 },
    NoteOff { note: u8 },
}

pub struct Scheduler {
    sample_rate: u32,
    samples_elapsed: u64,
    // Pattern-based (looping)
    patterns: Vec<PatternSlot>,
    // Queue-based (one-shot)
    queue: VecDeque<QueuedNote>,
    // Pending note-offs from patterns
    pending_offs: Vec<(u64, u8)>, // (absolute sample, note)
}

impl Scheduler {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            samples_elapsed: 0,
            patterns: Vec::new(),
            queue: VecDeque::new(),
            pending_offs: Vec::new(),
        }
    }

    pub fn set_pattern(&mut self, index: usize, slot: PatternSlot) {
        if index >= self.patterns.len() {
            self.patterns.resize_with(index + 1, || PatternSlot::empty(TICKS_PER_BEAT * 4));
        }
        self.patterns[index] = slot;
    }

    pub fn enqueue(&mut self, note: QueuedNote) {
        self.queue.push_back(note);
    }

    /// Enqueue a note to play immediately (at current sample position).
    pub fn enqueue_now(&mut self, note: u8, duration_samples: u64, gain: f32) {
        self.queue.push_back(QueuedNote {
            at_sample: self.samples_elapsed,
            note,
            duration_samples,
            gain,
        });
    }

    pub fn playhead(&self, pattern_index: usize) -> f32 {
        if let Some(pat) = self.patterns.get(pattern_index) {
            if !pat.active || pat.total_ticks == 0 {
                return 0.0;
            }
            let loop_samples = self.ticks_to_samples(pat.total_ticks);
            if loop_samples == 0 {
                return 0.0;
            }
            (self.samples_elapsed % loop_samples) as f32 / loop_samples as f32
        } else {
            0.0
        }
    }

    pub fn advance(&mut self, frames: usize) -> Vec<SchedulerEvent> {
        let mut events = Vec::new();
        let block_start = self.samples_elapsed;
        let block_end = block_start + frames as u64;

        // Process patterns
        for pat in &self.patterns {
            if !pat.active || pat.total_ticks == 0 {
                continue;
            }
            let loop_samples = self.ticks_to_samples(pat.total_ticks);
            if loop_samples == 0 {
                continue;
            }

            let loop_pos = block_start % loop_samples;

            for note_event in &pat.events {
                let note_sample = self.ticks_to_samples(note_event.tick);
                let note_off_sample = note_sample + self.ticks_to_samples(note_event.duration_ticks);

                // Note-on
                if in_range(note_sample, loop_pos, frames as u64, loop_samples) {
                    let offset = offset_in_block(note_sample, loop_pos, loop_samples);
                    events.push(SchedulerEvent {
                        frame_offset: offset as usize,
                        kind: EventKind::NoteOn {
                            note: note_event.note,
                            gain: note_event.gain,
                        },
                    });
                    // Schedule note-off
                    let off_abs = block_start + offset + self.ticks_to_samples(note_event.duration_ticks);
                    self.pending_offs.push((off_abs, note_event.note));
                }
            }
        }

        // Process queue
        while let Some(front) = self.queue.front() {
            if front.at_sample >= block_end {
                break;
            }
            let queued = self.queue.pop_front().unwrap();
            let offset = queued.at_sample.saturating_sub(block_start) as usize;
            events.push(SchedulerEvent {
                frame_offset: offset.min(frames.saturating_sub(1)),
                kind: EventKind::NoteOn {
                    note: queued.note,
                    gain: queued.gain,
                },
            });
            self.pending_offs.push((queued.at_sample + queued.duration_samples, queued.note));
        }

        // Process pending note-offs
        let mut remaining_offs = Vec::new();
        for (off_at, note) in self.pending_offs.drain(..) {
            if off_at >= block_start && off_at < block_end {
                let offset = (off_at - block_start) as usize;
                events.push(SchedulerEvent {
                    frame_offset: offset.min(frames.saturating_sub(1)),
                    kind: EventKind::NoteOff { note },
                });
            } else if off_at >= block_end {
                remaining_offs.push((off_at, note));
            }
            // else: already past, drop
        }
        self.pending_offs = remaining_offs;

        self.samples_elapsed += frames as u64;
        events.sort_by_key(|e| e.frame_offset);
        events
    }

    pub fn patterns(&self) -> &[PatternSlot] {
        &self.patterns
    }

    fn ticks_to_samples(&self, ticks: u32) -> u64 {
        // Assuming 120 BPM: 1 beat = 0.5s, TICKS_PER_BEAT ticks = 0.5s
        // samples = ticks * (sample_rate * 60) / (BPM * TICKS_PER_BEAT)
        // For now, hardcode 120 BPM
        let bpm = 120.0f64;
        let samples_per_tick = (self.sample_rate as f64 * 60.0) / (bpm * TICKS_PER_BEAT as f64);
        (ticks as f64 * samples_per_tick) as u64
    }
}

fn in_range(sample: u64, loop_pos: u64, block_len: u64, loop_len: u64) -> bool {
    let end = loop_pos + block_len;
    if end <= loop_len {
        sample >= loop_pos && sample < end
    } else {
        sample >= loop_pos || sample < (end % loop_len)
    }
}

fn offset_in_block(sample: u64, loop_pos: u64, loop_len: u64) -> u64 {
    if sample >= loop_pos {
        sample - loop_pos
    } else {
        (loop_len - loop_pos) + sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scheduler_produces_no_events() {
        let mut sched = Scheduler::new(48000);
        let events = sched.advance(128);
        assert!(events.is_empty());
    }

    #[test]
    fn pattern_triggers_note() {
        let mut sched = Scheduler::new(48000);
        sched.set_pattern(0, PatternSlot {
            events: vec![NoteEvent {
                tick: 0,
                note: 60,
                duration_ticks: 240,
                gain: 0.5,
            }],
            total_ticks: TICKS_PER_BEAT * 4,
            active: true,
        });

        let events = sched.advance(128);
        assert!(events.iter().any(|e| matches!(e.kind, EventKind::NoteOn { note: 60, .. })));
    }

    #[test]
    fn queue_triggers_immediately() {
        let mut sched = Scheduler::new(48000);
        sched.enqueue_now(72, 4800, 0.3);
        let events = sched.advance(128);
        assert!(events.iter().any(|e| matches!(e.kind, EventKind::NoteOn { note: 72, .. })));
    }

    #[test]
    fn chord_from_pattern() {
        let mut sched = Scheduler::new(48000);
        sched.set_pattern(0, PatternSlot {
            events: vec![
                NoteEvent { tick: 0, note: 60, duration_ticks: 240, gain: 0.3 },
                NoteEvent { tick: 0, note: 64, duration_ticks: 240, gain: 0.3 },
                NoteEvent { tick: 0, note: 67, duration_ticks: 240, gain: 0.3 },
            ],
            total_ticks: TICKS_PER_BEAT * 4,
            active: true,
        });

        let events = sched.advance(128);
        let note_ons: Vec<_> = events.iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOn { .. }))
            .collect();
        assert_eq!(note_ons.len(), 3);
    }
}
