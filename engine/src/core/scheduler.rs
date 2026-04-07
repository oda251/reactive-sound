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
    bpm: f64,
    samples_elapsed: u64,
    patterns: Vec<PatternSlot>,
    queue: VecDeque<QueuedNote>,
    pending_offs: Vec<(u64, u8)>,
    // Pre-allocated buffers to avoid hot-path allocation
    event_buf: Vec<SchedulerEvent>,
    off_buf: Vec<(u64, u8)>,
}

impl Scheduler {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            bpm: 120.0,
            samples_elapsed: 0,
            patterns: Vec::new(),
            queue: VecDeque::new(),
            pending_offs: Vec::with_capacity(64),
            event_buf: Vec::with_capacity(64),
            off_buf: Vec::with_capacity(64),
        }
    }

    #[allow(dead_code)]
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    pub fn set_pattern(&mut self, index: usize, slot: PatternSlot) {
        if index >= self.patterns.len() {
            self.patterns
                .resize_with(index + 1, || PatternSlot::empty(TICKS_PER_BEAT * 4));
        }
        self.patterns[index] = slot;
    }

    pub fn enqueue(&mut self, note: QueuedNote) {
        self.queue.push_back(note);
    }

    #[allow(dead_code)]
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

    /// Advance the scheduler and call `f` for each event in this block.
    pub fn advance(&mut self, frames: usize, mut f: impl FnMut(&SchedulerEvent)) {
        self.event_buf.clear();
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

                if in_range(note_sample, loop_pos, frames as u64, loop_samples) {
                    let offset = offset_in_block(note_sample, loop_pos, loop_samples);
                    self.event_buf.push(SchedulerEvent {
                        frame_offset: offset as usize,
                        kind: EventKind::NoteOn {
                            note: note_event.note,
                            gain: note_event.gain,
                        },
                    });
                    let off_abs =
                        block_start + offset + self.ticks_to_samples(note_event.duration_ticks);
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
            self.event_buf.push(SchedulerEvent {
                frame_offset: offset.min(frames.saturating_sub(1)),
                kind: EventKind::NoteOn {
                    note: queued.note,
                    gain: queued.gain,
                },
            });
            self.pending_offs
                .push((queued.at_sample + queued.duration_samples, queued.note));
        }

        // Process pending note-offs (swap buffers to avoid allocation)
        self.off_buf.clear();
        for &(off_at, note) in &self.pending_offs {
            if off_at >= block_start && off_at < block_end {
                let offset = (off_at - block_start) as usize;
                self.event_buf.push(SchedulerEvent {
                    frame_offset: offset.min(frames.saturating_sub(1)),
                    kind: EventKind::NoteOff { note },
                });
            } else if off_at >= block_end {
                self.off_buf.push((off_at, note));
            }
        }
        std::mem::swap(&mut self.pending_offs, &mut self.off_buf);

        self.samples_elapsed += frames as u64;
        self.event_buf.sort_by_key(|e| e.frame_offset);
        for event in &self.event_buf {
            f(event);
        }
    }

    #[allow(dead_code)]
    pub fn patterns(&self) -> &[PatternSlot] {
        &self.patterns
    }

    fn ticks_to_samples(&self, ticks: u32) -> u64 {
        let samples_per_tick =
            (self.sample_rate as f64 * 60.0) / (self.bpm * TICKS_PER_BEAT as f64);
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

    fn collect_events(sched: &mut Scheduler, frames: usize) -> Vec<SchedulerEvent> {
        let mut out = Vec::new();
        sched.advance(frames, |e| out.push(e.clone()));
        out
    }

    #[test]
    fn empty_scheduler_produces_no_events() {
        let mut sched = Scheduler::new(48000);
        let events = collect_events(&mut sched, 128);
        assert!(events.is_empty());
    }

    #[test]
    fn pattern_triggers_note() {
        let mut sched = Scheduler::new(48000);
        sched.set_pattern(
            0,
            PatternSlot {
                events: vec![NoteEvent {
                    tick: 0,
                    note: 60,
                    duration_ticks: 240,
                    gain: 0.5,
                }],
                total_ticks: TICKS_PER_BEAT * 4,
                active: true,
            },
        );

        let events = collect_events(&mut sched, 128);
        assert!(events.iter().any(|e| matches!(e.kind, EventKind::NoteOn { note: 60, .. })));
    }

    #[test]
    fn queue_triggers_immediately() {
        let mut sched = Scheduler::new(48000);
        sched.enqueue_now(72, 4800, 0.3);
        let events = collect_events(&mut sched, 128);
        assert!(events.iter().any(|e| matches!(e.kind, EventKind::NoteOn { note: 72, .. })));
    }

    #[test]
    fn chord_from_pattern() {
        let mut sched = Scheduler::new(48000);
        sched.set_pattern(
            0,
            PatternSlot {
                events: vec![
                    NoteEvent { tick: 0, note: 60, duration_ticks: 240, gain: 0.3 },
                    NoteEvent { tick: 0, note: 64, duration_ticks: 240, gain: 0.3 },
                    NoteEvent { tick: 0, note: 67, duration_ticks: 240, gain: 0.3 },
                ],
                total_ticks: TICKS_PER_BEAT * 4,
                active: true,
            },
        );

        let events = collect_events(&mut sched, 128);
        let note_ons: Vec<_> = events.iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOn { .. }))
            .collect();
        assert_eq!(note_ons.len(), 3);
    }
}
