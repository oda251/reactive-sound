use std::sync::mpsc;

use crate::core::effect::ImmediateAction;
use crate::core::scheduler::ParamValue;
use crate::core::{DspProcessor, EngineConfig, EventKind, Scheduler};
use crate::shell::command::Command;

pub struct Bridge {
    scheduler: Scheduler,
    dsp: DspProcessor,
    cmd_rx: mpsc::Receiver<Command>,
    channels: u16,
    block_size: usize,
    ring: Vec<f32>,
    ring_read: usize,
    ring_avail: usize,
    faust_buf: Vec<f32>,
}

const BLOCK_SIZE: usize = 128;

impl Bridge {
    pub fn new(
        scheduler: Scheduler,
        dsp: DspProcessor,
        cmd_rx: mpsc::Receiver<Command>,
        config: &EngineConfig,
    ) -> Self {
        let block_samples = BLOCK_SIZE * config.channels as usize;
        let ring_capacity = block_samples * 4;
        Self {
            scheduler,
            dsp,
            cmd_rx,
            channels: config.channels,
            block_size: BLOCK_SIZE,
            ring: vec![0.0; ring_capacity],
            ring_read: 0,
            ring_avail: 0,
            faust_buf: vec![0.0; block_samples],
        }
    }

    pub fn playhead(&self, pattern_index: usize) -> f32 {
        self.scheduler.playhead(pattern_index)
    }

    pub fn fill(&mut self, output: &mut [f32]) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                Command::SetPattern(idx, slot) => self.scheduler.set_pattern(idx, slot),
                Command::Enqueue(note) => self.scheduler.enqueue(note),
                Command::Immediate(action) => self.apply_immediate(action),
            }
        }

        let mut written = 0;

        while written < output.len() {
            while self.ring_avail > 0 && written < output.len() {
                output[written] = self.ring[self.ring_read];
                self.ring_read = (self.ring_read + 1) % self.ring.len();
                self.ring_avail -= 1;
                written += 1;
            }

            if written >= output.len() {
                break;
            }

            self.render_block_into_ring();
        }
    }

    fn apply_immediate(&mut self, action: ImmediateAction) {
        match action {
            ImmediateAction::NoteOn { note, gain } => {
                self.dsp.note_on(note, gain);
            }
            ImmediateAction::NoteOff { note } => self.dsp.note_off(note),
            ImmediateAction::SetParam(param, value) => self.dsp.set_voice_param(0, param, value),
        }
    }

    fn render_block_into_ring(&mut self) {
        let bs = self.block_size;
        let block_samples = bs * self.channels as usize;

        // Collect events into small vecs to break the borrow on self.scheduler
        let mut note_ons: Vec<(u8, f32, Vec<ParamValue>)> = Vec::new();
        let mut note_offs: Vec<u8> = Vec::new();
        let mut param_changes: Vec<ParamValue> = Vec::new();

        self.scheduler.advance(bs, |event| match &event.kind {
            EventKind::NoteOn {
                note,
                gain,
                overrides,
                ..
            } => note_ons.push((*note, *gain, overrides.clone())),
            EventKind::NoteOff { note, .. } => note_offs.push(*note),
            EventKind::ParamChange { change, .. } => param_changes.push(change.clone()),
        });

        for (note, gain, overrides) in &note_ons {
            let voice_idx = self.dsp.note_on(*note, *gain);
            for pv in overrides {
                self.dsp.set_voice_param(voice_idx, pv.param, pv.value);
            }
        }
        for note in &note_offs {
            self.dsp.note_off(*note);
        }
        for pv in &param_changes {
            // Global param changes apply to voice 0 for now
            // TODO: route via VoiceType when multi-DSP is implemented
            self.dsp.set_voice_param(0, pv.param, pv.value);
        }

        self.faust_buf[..block_samples].fill(0.0);
        self.dsp.render_interleaved(&mut self.faust_buf[..block_samples], bs);

        let write_start = (self.ring_read + self.ring_avail) % self.ring.len();
        let first_len = block_samples.min(self.ring.len() - write_start);
        let second_len = block_samples - first_len;

        self.ring[write_start..write_start + first_len]
            .copy_from_slice(&self.faust_buf[..first_len]);
        if second_len > 0 {
            self.ring[..second_len].copy_from_slice(&self.faust_buf[first_len..block_samples]);
        }

        self.ring_avail += block_samples;
    }
}
