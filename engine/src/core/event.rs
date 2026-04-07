use std::time::Instant;

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPress { timestamp: Instant },
}
