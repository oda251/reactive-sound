pub struct Mapper {
    current_tier: Tier,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tier {
    Idle,
    Slow,
    Medium,
    Fast,
}

pub struct MappedParams {
    pub pattern: &'static str,
    pub freq: f32,
    pub gain: f32,
    pub gate: bool,
    pub label: &'static str,
}

impl Mapper {
    pub fn new() -> Self {
        Self {
            current_tier: Tier::Idle,
        }
    }

    /// Returns Some(params) only when the tier changes.
    pub fn update(&mut self, wpm: f32) -> Option<MappedParams> {
        let new_tier = match wpm as u32 {
            0..=5 => Tier::Idle,
            6..=30 => Tier::Slow,
            31..=60 => Tier::Medium,
            _ => Tier::Fast,
        };

        if new_tier == self.current_tier {
            return None;
        }

        self.current_tier = new_tier;

        Some(match new_tier {
            // Idle: quiet ambient drone
            Tier::Idle => MappedParams {
                pattern: "o: sin 110 >> mul 0.03",
                freq: 110.0,
                gain: 0.0,
                gate: false,
                label: "Idle",
            },
            // Slow: sparse, low notes
            Tier::Slow => MappedParams {
                pattern: "o: seq 48 0 0 0 0 0 0 0 >> sawsynth 0.05 0.5 >> mul 0.15",
                freq: 165.0,
                gain: 0.15,
                gate: true,
                label: "Slow",
            },
            // Medium: melodic sequence
            Tier::Medium => MappedParams {
                pattern: "o: seq 60 0 67 0 72 0 67 0 >> sawsynth 0.01 0.1 >> mul 0.2",
                freq: 330.0,
                gain: 0.25,
                gate: true,
                label: "Medium",
            },
            // Fast: dense, high energy
            Tier::Fast => MappedParams {
                pattern: "o: seq 72 67 75 60 72 63 67 60 >> sawsynth 0.005 0.05 >> mul 0.25",
                freq: 440.0,
                gain: 0.35,
                gate: true,
                label: "Fast",
            },
        })
    }
}
