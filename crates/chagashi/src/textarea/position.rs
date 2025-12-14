#[derive(Default, Clone, Copy, Debug, PartialEq)]
/// 2D cursor position within a text buffer (x = column, y = row).
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
