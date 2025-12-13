pub trait Termable {
    fn size(&self) -> Result<(u16, u16), std::io::Error>;
    fn hide_cursor(&self) -> Result<(), std::io::Error>;
    fn show_cursor(&self) -> Result<(), std::io::Error>;
    fn enable_raw_mode(&self) -> Result<(), std::io::Error>;
    fn disable_raw_mode(&self) -> Result<(), std::io::Error>;
    fn print(&self, v: &str) -> Result<(), std::io::Error>;
    fn enter_alt_screen(&self) -> Result<(), std::io::Error>;
    fn leave_alt_screen(&self) -> Result<(), std::io::Error>;
    fn enable_mouse_capture(&self) -> Result<(), std::io::Error>;
    fn disable_mouse_capture(&self) -> Result<(), std::io::Error>;
    fn move_to_column(&self, y: u16) -> Result<(), std::io::Error>;
    fn move_to(&self, x: u16, y: u16) -> Result<(), std::io::Error>;
    fn clear_all(&self) -> Result<(), std::io::Error>;
    fn clear_current_line(&self) -> Result<(), std::io::Error>;
    fn clear_current_line_and_move_previous(&self) -> Result<(), std::io::Error>;
    /// Save current cursor position (for non-alt-screen redraw anchoring).
    fn save_cursor_position(&self) -> Result<(), std::io::Error>;
    /// Restore previously saved cursor position.
    fn restore_cursor_position(&self) -> Result<(), std::io::Error>;
    /// Clear from current cursor position down to the end of the screen.
    fn clear_from_cursor_down(&self) -> Result<(), std::io::Error>;
}
