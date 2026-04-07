use glicol::Engine as GlicolEngine;

const BLOCK_SIZE: usize = 128;
const EMPTY_INPUT: &[&[f32]] = &[];

pub struct Pattern {
    engine: GlicolEngine<BLOCK_SIZE>,
}

impl Pattern {
    pub fn new(sample_rate: u32) -> Self {
        let mut engine = GlicolEngine::<BLOCK_SIZE>::new();
        engine.set_sr(sample_rate as usize);
        engine.update_with_code("");
        Self { engine }
    }

    pub fn update_code(&mut self, code: &str) {
        self.engine.update_with_code(code);
    }

    pub fn render_interleaved(&mut self, out: &mut [f32], output_channels: u16) -> usize {
        let (buf, _) = self.engine.next_block(EMPTY_INPUT.to_vec());
        let src_channels = buf.len();
        let out_ch = output_channels as usize;
        let frames = BLOCK_SIZE.min(out.len() / out_ch.max(1));

        for frame in 0..frames {
            for ch in 0..out_ch {
                let value = if ch < src_channels {
                    buf[ch].get(frame).copied().unwrap_or(0.0)
                } else if src_channels > 0 {
                    buf[0].get(frame).copied().unwrap_or(0.0)
                } else {
                    0.0
                };
                out[frame * out_ch + ch] = value;
            }
        }

        frames
    }

    pub fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_on_empty_pattern() {
        let mut pat = Pattern::new(48000);
        let mut buf = vec![0.0f32; BLOCK_SIZE * 2];
        pat.render_interleaved(&mut buf, 2);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn produces_audio_with_pattern() {
        let mut pat = Pattern::new(48000);
        pat.update_code("o: sin 440");
        let mut buf = vec![0.0f32; BLOCK_SIZE * 2];
        pat.render_interleaved(&mut buf, 2);
        let max = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0, "expected non-silent output, got max={max}");
    }
}
