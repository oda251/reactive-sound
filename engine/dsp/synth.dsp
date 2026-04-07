import("stdfaust.lib");

freq = hslider("freq", 440, 20, 20000, 1);
gain = hslider("gain", 0.5, 0, 1, 0.01);
gate = button("gate");

envelope = en.adsr(0.01, 0.1, 0.7, 0.3, gate);
process = os.sawtooth(freq) * envelope * gain <: _, _;
