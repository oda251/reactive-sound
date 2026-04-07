use std::sync::mpsc;

use crate::core::dsp::PARAM_FREQ;
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

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

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

    pub fn playhead(&self) -> f32 {
        self.scheduler.playhead()
    }

    pub fn fill(&mut self, output: &mut [f32]) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                Command::SetScore(score) => self.scheduler.set_score(score),
                Command::SetDspParam(idx, val) => self.dsp.set_param(idx, val),
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

    fn render_block_into_ring(&mut self) {
        let bs = self.block_size;
        let block_samples = bs * self.channels as usize;

        // Advance scheduler and process events
        let events = self.scheduler.advance(bs);
        for event in &events {
            match &event.kind {
                EventKind::NoteOn { note } => {
                    self.dsp.set_param(PARAM_FREQ, midi_to_freq(*note));
                    self.dsp.set_param(crate::core::dsp::PARAM_GATE, 1.0);
                }
                EventKind::NoteOff { .. } => {
                    self.dsp.set_param(crate::core::dsp::PARAM_GATE, 0.0);
                }
            }
        }

        // Render Faust
        self.faust_buf[..block_samples].fill(0.0);
        self.dsp.render_interleaved(&mut self.faust_buf[..block_samples], bs);

        // Write to ring
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
