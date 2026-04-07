use std::sync::mpsc;

use crate::core::{DspProcessor, EngineConfig, Pattern};
use crate::shell::command::Command;

pub struct Bridge {
    pattern: Pattern,
    dsp: DspProcessor,
    cmd_rx: mpsc::Receiver<Command>,
    channels: u16,
    block_samples: usize,
    ring: Vec<f32>,
    ring_read: usize,
    ring_avail: usize,
    glicol_buf: Vec<f32>,
    faust_buf: Vec<f32>,
}

impl Bridge {
    pub fn new(
        pattern: Pattern,
        dsp: DspProcessor,
        cmd_rx: mpsc::Receiver<Command>,
        config: &EngineConfig,
    ) -> Self {
        let block_samples = pattern.block_size() * config.channels as usize;
        let ring_capacity = block_samples * 4;
        Self {
            pattern,
            dsp,
            cmd_rx,
            channels: config.channels,
            block_samples,
            ring: vec![0.0; ring_capacity],
            ring_read: 0,
            ring_avail: 0,
            glicol_buf: vec![0.0; block_samples],
            faust_buf: vec![0.0; block_samples],
        }
    }

    pub fn fill(&mut self, output: &mut [f32]) {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                Command::UpdatePattern(code) => self.pattern.update_code(&code),
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
        let bs = self.block_samples;

        self.glicol_buf[..bs].fill(0.0);
        self.pattern.render_interleaved(&mut self.glicol_buf[..bs], self.channels);

        self.faust_buf[..bs].fill(0.0);
        self.dsp.render_interleaved(&mut self.faust_buf[..bs], bs / self.channels as usize);

        // Write mixed output to ring in two linear passes (avoids per-sample modulo)
        let write_start = (self.ring_read + self.ring_avail) % self.ring.len();
        let first_len = bs.min(self.ring.len() - write_start);
        let second_len = bs - first_len;

        for i in 0..first_len {
            self.ring[write_start + i] = self.glicol_buf[i] + self.faust_buf[i];
        }
        for i in 0..second_len {
            self.ring[i] = self.glicol_buf[first_len + i] + self.faust_buf[first_len + i];
        }

        self.ring_avail += bs;
    }
}
