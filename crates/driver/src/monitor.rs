#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monitor {
    pub(super) name: Box<str>,
    pub(super) bounds: MonitorBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
