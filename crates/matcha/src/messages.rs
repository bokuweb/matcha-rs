use crate::Msg;

/// Quit is a special command that tells the Bubble Tea program to exit.
pub fn quit() -> Msg {
    Box::new(QuitMsg)
}

/// quitMsg in an internal message signals that the program should quit. You can
/// send a quitMsg with Quit.
pub struct QuitMsg;
