mod core;
mod shell;

use std::env;

pub use crate::core::dsp::{PARAM_FREQ, PARAM_GAIN, PARAM_GATE};
pub use crate::core::effect::{AccumulativeEffect, ImmediateAction, ImmediateEffect, InputEffect};
pub use crate::core::event::InputEvent;
pub use crate::core::scheduler::{
    EventKind, NoteEvent, ParamEvent, ParamValue, PatternSlot, QueuedNote, VoiceType,
    TICKS_PER_BEAT,
};
pub use crate::core::EngineConfig;
use crate::shell::audio;
use crate::shell::command::Command;

pub struct Engine {
    audio: audio::AudioOutput,
}

impl Engine {
    pub fn start_default() -> anyhow::Result<Self> {
        Self::start(config_from_env())
    }

    pub fn start(config: EngineConfig) -> anyhow::Result<Self> {
        let audio = audio::AudioOutput::start(&config)?;
        Ok(Self { audio })
    }

    pub fn set_pattern(&mut self, index: usize, slot: PatternSlot) -> Result<(), EngineError> {
        self.send(Command::SetPattern(index, slot))
    }

    pub fn enqueue(&mut self, note: QueuedNote) -> Result<(), EngineError> {
        self.send(Command::Enqueue(note))
    }

    pub fn send_immediate(&mut self, action: ImmediateAction) -> Result<(), EngineError> {
        self.send(Command::Immediate(action))
    }

    pub fn playhead(&self) -> f32 {
        self.audio.playhead()
    }

    pub fn start_time(&self) -> std::time::Instant {
        self.audio.start_time()
    }

    fn send(&mut self, cmd: Command) -> Result<(), EngineError> {
        self.audio
            .send(cmd)
            .map_err(|_| EngineError::ChannelFull)
    }
}

#[derive(Debug)]
pub enum EngineError {
    ChannelFull,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelFull => write!(f, "command ring buffer full"),
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
