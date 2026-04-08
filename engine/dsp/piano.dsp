import("stdfaust.lib");

freq = hslider("freq", 440, 20, 20000, 1);
gain = hslider("gain", 0.5, 0, 1, 0.01);
gate = button("gate");

// Sharp attack, long decay envelope (piano-like)
envelope = en.adsr(0.001, 1.5, 0.0, 0.3, gate);

// Additive synthesis: fundamental + harmonics with decreasing amplitude
harmonic(n) = os.osc(freq * n) * (1.0 / n);

tone = harmonic(1)         // fundamental
     + harmonic(2) * 0.7   // octave
     + harmonic(3) * 0.3   // fifth above octave
     + harmonic(4) * 0.15  // 2nd octave
     + harmonic(5) * 0.08  // major third above 2nd octave
     + harmonic(6) * 0.04; // 3rd octave partial

// Brightness decay: high frequencies fade faster than low
brightness = en.adsr(0.001, 0.6, 0.2, 0.1, gate);
filtered = tone : fi.lowpass(2, 800 + 4000 * brightness);

process = filtered * envelope * gain <: _, _;
