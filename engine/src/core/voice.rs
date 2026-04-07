const MAX_VOICES: usize = 8;

#[derive(Clone, Copy)]
struct Voice {
    note: Option<u8>,
    age: u64,
    active: bool,
}

pub struct VoiceAllocator {
    voices: [Voice; MAX_VOICES],
    counter: u64,
}

impl VoiceAllocator {
    pub fn new() -> Self {
        Self {
            voices: [Voice { note: None, age: 0, active: false }; MAX_VOICES],
            counter: 0,
        }
    }

    pub fn num_voices(&self) -> usize {
        MAX_VOICES
    }

    /// Allocate a voice for a note. Returns the voice index.
    pub fn note_on(&mut self, note: u8) -> usize {
        self.counter += 1;

        // Reuse voice already playing this note
        if let Some(idx) = self.voices.iter().position(|v| v.note == Some(note)) {
            self.voices[idx].age = self.counter;
            self.voices[idx].active = true;
            return idx;
        }

        if let Some(idx) = self.voices.iter().position(|v| v.note.is_none()) {
            self.voices[idx] = Voice { note: Some(note), age: self.counter, active: true };
            return idx;
        }

        let idx = self.voices
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| v.age)
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.voices[idx] = Voice { note: Some(note), age: self.counter, active: true };
        idx
    }

    /// Release the voice playing this note. Returns the voice index if found.
    pub fn note_off(&mut self, note: u8) -> Option<usize> {
        if let Some(idx) = self.voices.iter().position(|v| v.note == Some(note)) {
            self.voices[idx].note = None;
            // Keep active=true so the Faust ADSR release phase can complete.
            // The voice will be marked inactive when reused by note_on.
            Some(idx)
        } else {
            None
        }
    }

    pub fn is_active(&self, idx: usize) -> bool {
        idx < MAX_VOICES && self.voices[idx].active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_free_voice() {
        let mut alloc = VoiceAllocator::new();
        let v0 = alloc.note_on(60);
        let v1 = alloc.note_on(64);
        assert_ne!(v0, v1);
    }

    #[test]
    fn reuses_same_note() {
        let mut alloc = VoiceAllocator::new();
        let v0 = alloc.note_on(60);
        let v1 = alloc.note_on(60);
        assert_eq!(v0, v1);
    }

    #[test]
    fn releases_voice() {
        let mut alloc = VoiceAllocator::new();
        let v0 = alloc.note_on(60);
        let released = alloc.note_off(60);
        assert_eq!(released, Some(v0));
        // Voice is now free, next note gets the same slot
        let v1 = alloc.note_on(64);
        assert_eq!(v0, v1);
    }

    #[test]
    fn steals_oldest_when_full() {
        let mut alloc = VoiceAllocator::new();
        for i in 0..8 {
            alloc.note_on(60 + i as u8);
        }
        // All 8 voices occupied, note_on should steal oldest (note 60)
        let stolen = alloc.note_on(80);
        assert_eq!(stolen, 0); // voice 0 was oldest
    }
}
