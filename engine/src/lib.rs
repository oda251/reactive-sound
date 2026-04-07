mod core;
mod shell;

use std::env;

pub use crate::core::dsp::{PARAM_FREQ, PARAM_GAIN, PARAM_GATE};
pub use crate::core::effect::{AccumulativeEffect, ImmediateAction, ImmediateEffect, InputEffect};
pub use crate::core::event::InputEvent;
pub use crate::core::scheduler::{EventKind, NoteEvent, PatternSlot, QueuedNote, TICKS_PER_BEAT};
pub use crate::core::EngineConfig;
use crate::shell::audio;
use crate::shell::command::Command;

pub struct Engine {
    audio: audio::AudioOutput,
}

impl Engine {
    pub fn start_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::start(config_from_env())
    }

    pub fn start(config: EngineConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let audio = audio::AudioOutput::start(&config)?;
        Ok(Self { audio })
    }

    pub fn set_pattern(&self, index: usize, slot: PatternSlot) -> Result<(), EngineError> {
        self.send(Command::SetPattern(index, slot))
    }

    pub fn enqueue(&self, note: QueuedNote) -> Result<(), EngineError> {
        self.send(Command::Enqueue(note))
    }

    pub fn send_immediate(&self, action: ImmediateAction) -> Result<(), EngineError> {
        self.send(Command::Immediate(action))
    }

    pub fn playhead(&self) -> f32 {
        self.audio.playhead()
    }

    fn send(&self, cmd: Command) -> Result<(), EngineError> {
        self.audio
            .send(cmd)
            .map_err(|_| EngineError::AudioThreadStopped)
    }
}

#[derive(Debug)]
pub enum EngineError {
    AudioThreadStopped,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AudioThreadStopped => write!(f, "audio thread has stopped"),
        }
    }
}

impl std::error::Error for EngineError {}

fn config_from_env() -> EngineConfig {
    let mut config = EngineConfig::default();

    if let Ok(val) = env::var("RBGM_SAMPLE_RATE") {
        if let Ok(sr) = val.parse() {
            config.sample_rate = Some(sr);
        }
    }
    if let Ok(val) = env::var("RBGM_DEVICE") {
        config.device_name = Some(val);
    }

    config
}
