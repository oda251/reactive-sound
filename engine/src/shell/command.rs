use crate::core::effect::ImmediateAction;
use crate::core::scheduler::{PatternSlot, QueuedNote};

pub enum Command {
    SetPattern(usize, PatternSlot),
    Enqueue(QueuedNote),
    Immediate(ImmediateAction),
}
