#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeWalkControl {
    Continue,
    SkipChildren,
    Stop,
}
