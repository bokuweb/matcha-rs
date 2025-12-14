/// Abstraction over terminal backends used by [`crate::Program`].
///
/// Most users will rely on the default backend, but this trait makes it possible
/// to inject a fake terminal for tests or to integrate other terminal libraries.
pub trait Termable {
    /// Return the terminal size in cells `(width, height)`.
    fn size(&self) -> Result<(u16, u16), std::io::Error>;
    /// Hide the cursor.
    fn hide_cursor(&self) -> Result<(), std::io::Error>;
    /// Show the cursor.
    fn show_cursor(&self) -> Result<(), std::io::Error>;
    /// Enable raw mode.
    fn enable_raw_mode(&self) -> Result<(), std::io::Error>;
    /// Disable raw mode.
    fn disable_raw_mode(&self) -> Result<(), std::io::Error>;
    /// Print raw bytes/text to the terminal.
    fn print(&self, v: &str) -> Result<(), std::io::Error>;
    /// Enter the alternate screen buffer.
    fn enter_alt_screen(&self) -> Result<(), std::io::Error>;
    /// Leave the alternate screen buffer.
    fn leave_alt_screen(&self) -> Result<(), std::io::Error>;
    /// Enable mouse capture.
    fn enable_mouse_capture(&self) -> Result<(), std::io::Error>;
    /// Disable mouse capture.
    fn disable_mouse_capture(&self) -> Result<(), std::io::Error>;
    /// Move cursor to a column.
    fn move_to_column(&self, y: u16) -> Result<(), std::io::Error>;
    /// Move cursor to `(x, y)`.
    fn move_to(&self, x: u16, y: u16) -> Result<(), std::io::Error>;
    /// Query current cursor position.
    fn cursor_position(&self) -> Result<(u16, u16), std::io::Error>;
    /// Clear the entire screen.
    fn clear_all(&self) -> Result<(), std::io::Error>;
    /// Clear the current line.
    fn clear_current_line(&self) -> Result<(), std::io::Error>;
    /// Clear current line and move to previous line.
    fn clear_current_line_and_move_previous(&self) -> Result<(), std::io::Error>;
    /// Save current cursor position (for non-alt-screen redraw anchoring).
    fn save_cursor_position(&self) -> Result<(), std::io::Error>;
    /// Restore previously saved cursor position.
    fn restore_cursor_position(&self) -> Result<(), std::io::Error>;
    /// Clear from current cursor position down to the end of the screen.
    fn clear_from_cursor_down(&self) -> Result<(), std::io::Error>;
}
