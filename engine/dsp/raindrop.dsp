import("stdfaust.lib");

freq = hslider("freq", 2000, 20, 20000, 1);
gain = hslider("gain", 0.5, 0, 1, 0.01);
gate = button("gate");

// Sharp impact envelope
impact = en.adsr(0.0001, 0.08, 0.0, 0.15, gate);

// Pitched resonance: sine with quick pitch drop simulates water surface tension
pitch_env = en.adsr(0.0001, 0.03, 0.0, 0.01, gate);
drop_tone = os.osc(freq * (1.0 + pitch_env * 0.5)) * 0.4;

// Noise splash: band-passed noise around the freq
splash = no.noise : fi.resonbp(freq, 5, 1.0) * 0.6;

// Tail: lower-frequency filtered noise for ambient ring
tail_env = en.adsr(0.0001, 0.15, 0.0, 0.3, gate);
tail = no.noise : fi.lowpass(2, freq * 0.3) * tail_env * 0.15;

// Mix
raindrop = (drop_tone + splash) * impact + tail;

process = raindrop * gain <: _, _;
