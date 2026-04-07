use crate::core::Score;

pub enum Command {
    SetScore(Score),
    SetDspParam(i32, f32),
}
