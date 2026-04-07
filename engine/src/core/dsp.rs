#[allow(
    clippy::all,
    unused_parens,
    non_snake_case,
    non_camel_case_types,
    dead_code,
    unused_variables,
    unused_mut,
    non_upper_case_globals
)]
mod faust_generated {
    include!(concat!(env!("OUT_DIR"), "/faust_synth.rs"));
}
use faust_generated::*;

pub const PARAM_FREQ: i32 = 0;
pub const PARAM_GAIN: i32 = 1;
pub const PARAM_GATE: i32 = 2;

pub struct DspProcessor {
    synth: FaustSynth,
    output_bufs: [Vec<f32>; 2],
}

impl DspProcessor {
    pub fn new(sample_rate: u32, max_block_size: usize) -> Self {
        let mut synth = FaustSynth::new();
        synth.init(sample_rate as i32);
        Self {
            synth,
            output_bufs: [vec![0.0; max_block_size], vec![0.0; max_block_size]],
        }
    }

    pub fn set_param(&mut self, index: i32, value: f32) {
        self.synth.set_param(ParamIndex(index), value);
    }

    pub fn render_interleaved(&mut self, out: &mut [f32], frames: usize) -> usize {
        let frames = frames.min(out.len() / 2);

        let [buf0, buf1] = &mut self.output_bufs;
        buf0[..frames].fill(0.0);
        buf1[..frames].fill(0.0);

        let inputs: &[&[f32]] = &[];
        self.synth.compute(frames, inputs, &mut [&mut buf0[..frames], &mut buf1[..frames]]);

        for i in 0..frames {
            out[i * 2] = buf0[i];
            out[i * 2 + 1] = buf1[i];
        }

        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_without_gate() {
        let mut dsp = DspProcessor::new(48000, 128);
        dsp.set_param(PARAM_FREQ, 440.0);
        dsp.set_param(PARAM_GAIN, 0.5);
        let mut buf = vec![0.0f32; 256];
        dsp.render_interleaved(&mut buf, 128);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn produces_audio_with_gate() {
        let mut dsp = DspProcessor::new(48000, 128);
        dsp.set_param(PARAM_FREQ, 440.0);
        dsp.set_param(PARAM_GAIN, 0.5);
        dsp.set_param(PARAM_GATE, 1.0);
        let mut buf = vec![0.0f32; 256];
        dsp.render_interleaved(&mut buf, 128);
        let max = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0, "expected non-silent output, got max={max}");
    }
}
