/// Trait for a polyphonic synthesizer backend.
/// DspProcessor implements this with Faust voices.
/// Other implementations (e.g., sample player) can replace it.
pub trait Synth {
    fn note_on(&mut self, note: u8, gain: f32) -> usize;
    fn note_off(&mut self, note: u8);
    fn set_voice_param(&mut self, voice: usize, param: i32, value: f32);
    fn render_interleaved(&mut self, out: &mut [f32], frames: usize) -> usize;
}
