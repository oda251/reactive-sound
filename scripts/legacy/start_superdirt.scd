s.options.numBuffers = 4096;
s.options.memSize = 8192 * 32; // 256KB -> 256MB
s.waitForBoot {
    ~dirt = SuperDirt(2, s);
    ~dirt.loadSoundFiles;
    s.sync;
    ~dirt.start(57120, [0, 0]);
    ">>> SuperDirt started! <<<".postln;
};
