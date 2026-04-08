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
mod faust_synth {
    include!(concat!(env!("OUT_DIR"), "/faust_synth.rs"));
}

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
mod faust_piano {
    include!(concat!(env!("OUT_DIR"), "/faust_piano.rs"));
}

use faust_piano::*;

pub const PARAM_FREQ: i32 = 0;
pub const PARAM_GAIN: i32 = 1;
pub const PARAM_GATE: i32 = 2;
use crate::core::voice::VoiceAllocator;

pub struct DspProcessor {
    voices: Vec<FaustPiano>,
    allocator: VoiceAllocator,
    // Per-voice output buffers
    voice_bufs: Vec<[Vec<f32>; 2]>,
    // Mixed output
    mix_buf: [Vec<f32>; 2],
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

impl DspProcessor {
    pub fn new(sample_rate: u32, max_block_size: usize) -> Self {
        let allocator = VoiceAllocator::new();
        let num_voices = allocator.num_voices();

        let mut voices = Vec::with_capacity(num_voices);
        for _ in 0..num_voices {
            let mut synth = FaustPiano::new();
            synth.init(sample_rate as i32);
            voices.push(synth);
        }

        let voice_bufs = (0..num_voices)
            .map(|_| [vec![0.0; max_block_size], vec![0.0; max_block_size]])
            .collect();

        Self {
            voices,
            allocator,
            voice_bufs,
            mix_buf: [vec![0.0; max_block_size], vec![0.0; max_block_size]],
        }
    }

    /// Returns the voice index that was allocated.
    pub fn note_on(&mut self, note: u8, gain: f32) -> usize {
        let idx = self.allocator.note_on(note);
        self.voices[idx].set_param(ParamIndex(PARAM_FREQ), midi_to_freq(note));
        self.voices[idx].set_param(ParamIndex(PARAM_GAIN), gain);
        self.voices[idx].set_param(ParamIndex(PARAM_GATE), 1.0);
        idx
    }

    pub fn note_off(&mut self, note: u8) {
        if let Some(idx) = self.allocator.note_off(note) {
            self.voices[idx].set_param(ParamIndex(PARAM_GATE), 0.0);
        }
    }

    pub fn set_voice_param(&mut self, voice: usize, param: i32, value: f32) {
        if voice < self.voices.len() {
            self.voices[voice].set_param(ParamIndex(param), value);
        }
    }

    pub fn render_interleaved(&mut self, out: &mut [f32], frames: usize) -> usize {
        let frames = frames.min(out.len() / 2);
        let inputs: &[&[f32]] = &[];

        // Clear mix buffer
        self.mix_buf[0][..frames].fill(0.0);
        self.mix_buf[1][..frames].fill(0.0);

        // Render only active voices
        for (i, synth) in self.voices.iter_mut().enumerate() {
            if !self.allocator.is_active(i) {
                continue;
            }
            let [b0, b1] = &mut self.voice_bufs[i];
            b0[..frames].fill(0.0);
            b1[..frames].fill(0.0);
            synth.compute(frames, inputs, &mut [&mut b0[..frames], &mut b1[..frames]]);

            for j in 0..frames {
                self.mix_buf[0][j] += b0[j];
                self.mix_buf[1][j] += b1[j];
            }
        }

        // Interleave
        for i in 0..frames {
            out[i * 2] = self.mix_buf[0][i];
            out[i * 2 + 1] = self.mix_buf[1][i];
        }

        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_without_notes() {
        let mut dsp = DspProcessor::new(48000, 128);
        let mut buf = vec![0.0f32; 256];
        dsp.render_interleaved(&mut buf, 128);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn single_note() {
        let mut dsp = DspProcessor::new(48000, 128);
        dsp.note_on(69, 0.5); // A4
        let mut buf = vec![0.0f32; 256];
        dsp.render_interleaved(&mut buf, 128);
        let max = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0);
    }

    #[test]
    fn chord() {
        let mut dsp = DspProcessor::new(48000, 128);
        dsp.note_on(60, 0.3); // C4
        dsp.note_on(64, 0.3); // E4
        dsp.note_on(67, 0.3); // G4
        let mut buf = vec![0.0f32; 256];
        dsp.render_interleaved(&mut buf, 128);
        let max = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max > 0.0);
    }

    #[test]
    fn note_off_silences() {
        let mut dsp = DspProcessor::new(48000, 128);
        dsp.note_on(69, 0.5);
        // Render a few blocks to let envelope open
        let mut buf = vec![0.0f32; 256];
        dsp.render_interleaved(&mut buf, 128);
        // Note off
        dsp.note_off(69);
        // Render many blocks to let envelope close
        for _ in 0..200 {
            dsp.render_interleaved(&mut buf, 128);
        }
        let max = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(max < 0.01, "expected near-silence after note off, got max={max}");
    }
}
