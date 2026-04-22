import("stdfaust.lib");

freq = hslider("freq", 1000, 20, 20000, 1);
gain = hslider("gain", 0.5, 0, 1, 0.01);
gate = button("gate");

// Very short percussive envelope — sharp attack, fast decay, no sustain
envelope = en.adsr(0.0005, 0.015, 0.0, 0.01, gate);

// Band-passed noise burst for a clicky, percussive sound
click = no.noise : fi.bandpass(2, freq * 0.8, freq * 1.2) * 3.0;

process = click * envelope * gain <: _, _;
