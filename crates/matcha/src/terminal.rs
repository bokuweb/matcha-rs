use crossterm::{
    cursor::{self, MoveTo, MoveToColumn},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

/// The default [`crate::Termable`] implementation backed by `crossterm`.
pub struct DefaultTerminal;

impl crate::termable::Termable for DefaultTerminal {
    fn size(&self) -> Result<(u16, u16), std::io::Error> {
        crossterm::terminal::size()
    }

    fn hide_cursor(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), cursor::Hide)
    }

    fn show_cursor(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), cursor::Show)
    }

    fn enable_raw_mode(&self) -> Result<(), std::io::Error> {
        enable_raw_mode()
    }

    fn disable_raw_mode(&self) -> Result<(), std::io::Error> {
        disable_raw_mode()
    }

    fn print(&self, v: &str) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), Print(v))
    }

    fn enter_alt_screen(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), EnterAlternateScreen)
    }

    fn leave_alt_screen(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), LeaveAlternateScreen)
    }

    fn enable_mouse_capture(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)
    }

    fn disable_mouse_capture(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)
    }

    fn move_to_column(&self, y: u16) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), MoveToColumn(y),)
    }

    fn move_to(&self, x: u16, y: u16) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), MoveTo(x, y),)
    }

    fn cursor_position(&self) -> Result<(u16, u16), std::io::Error> {
        crossterm::cursor::position()
    }

    fn clear_all(&self) -> Result<(), std::io::Error> {
        execute!(
            std::io::stdout(),
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )
    }

    fn clear_current_line(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), Clear(ClearType::CurrentLine),)
    }

    fn clear_current_line_and_move_previous(&self) -> Result<(), std::io::Error> {
        execute!(
            std::io::stdout(),
            cursor::MoveToPreviousLine(1),
            Clear(ClearType::CurrentLine)
        )
    }

    fn save_cursor_position(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), cursor::SavePosition)
    }

    fn restore_cursor_position(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), cursor::RestorePosition)
    }

    fn clear_from_cursor_down(&self) -> Result<(), std::io::Error> {
        execute!(std::io::stdout(), Clear(ClearType::FromCursorDown))
    }
}
