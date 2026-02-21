//! `matcha` is a small terminal UI framework inspired by Bubble Tea.
//!
//! It provides a simple architecture:
//! - Implement [`Model`] to hold state and render a view.
//! - Return [`Cmd`]s from `init`/`update` to perform side effects.
//! - Send messages ([`Msg`]) back into the update loop.
//!
//! This crate focuses on the runtime/event-loop and basic formatting helpers.
//! Higher-level UI components live in the companion crate `chagashi`.

mod dyn_model;
mod extension;
mod formatter;
mod key;
mod messages;
mod termable;
mod terminal;

pub use dyn_model::{boxed, DynModel};
pub use extension::*;
pub use formatter::*;
pub use key::*;
pub use messages::*;
pub use termable::Termable;
use terminal::DefaultTerminal;

pub extern crate crossterm;

use std::{any::Any, fmt::Display};

use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

// re-exports
pub use crokey::*;
pub use crossterm::{cursor, event::*, style::*};

use futures::{future::FutureExt, StreamExt};

/// Msg contain data from the result of a IO operation. Msgs trigger the update
/// function and, henceforth, the UI.
pub type Msg = Box<dyn Any + Send>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// Input provided to [`Model::init`].
///
/// This is typically used to initialize layout based on the initial terminal size.
pub struct InitInput {
    /// Initial terminal size `(width, height)` in cells.
    pub size: (u16, u16),
}

/// Model contains the program's state as well as its core functions.
pub trait Model: Sized {
    /// Init is the first function that will be called. It returns an optional
    /// initial command. To not perform an initial command return nil.
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        (self, None)
    }

    /// Update is called when a message is received. Use it to inspect messages
    /// and, in response, update the model and/or send a command.
    /// # Example
    ///
    /// ```ignore
    /// fn update(self, msg: Msg) -> (Self, Option<Cmd>) {
    ///    if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
    ///        if let KeyModifiers::CONTROL = key_event.modifiers {
    ///            match key_event.code {
    ///                KeyCode::Char('c') => return (self, Some(Cmd::sync(Box::new(quit)))),
    ///                _ => return (self, None),
    ///            }
    ///        }
    ///    };
    ///    (self, None)
    /// }
    /// ```
    ///
    fn update(self, _msg: &Msg) -> (Self, Option<Cmd>) {
        (self, None)
    }

    /// An asynchronous function that can execute commands received from either `init` or `update`.
    /// This function needs to accept a [`Cmd`] and return an `Option<Cmd>`.
    /// If not needed, implementation is not required.
    /// # Example
    ///
    /// ```ignore
    /// fn execute(
    ///     _ext: Extensions,
    ///     AsyncCmd(cmd): AsyncCmd,
    /// ) -> impl std::future::Future<Output = Option<Cmd>> + Send {
    ///     async move {
    ///         let msg = cmd();
    ///         if msg.downcast_ref::<AsyncMsg>().is_some() {
    ///             tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    ///             return Some(Cmd::sync(Box::new(DoneMsg)));
    ///         }
    ///         None
    ///     }
    /// }
    /// ```
    ///
    fn execute(
        _ext: Extensions,
        _cmd: AsyncCmd,
    ) -> impl std::future::Future<Output = Option<Cmd>> + Send {
        async { None }
    }

    /// View renders the program's UI, which is just a string. The view is
    /// rendered after every Update.
    fn view(&self) -> impl Display;
}

/// A boxed function or closure that performs computations and optionally dispatches messages.
/// All commands are processed in their own threads, so blocking commands are totally fine.
/// Frequently, data needs to be passed to commands. Since commands take no arguments,
/// a common solution to this is to build constructor functions.
///
/// # Example
///
/// ```ignore
/// // a constructor function
/// fn make_request_command(url: &str) -> Command {
///     // it's okay to block since commands are multi threaded
///     let text_response = reqwest::blocking::get(url).unwrap().text().unwrap();
///
///     // the command itself
///     Box::new(move || Some(Box::new(HttpResponse(text_response))))
/// }
/// ```
pub type CmdFn = Box<dyn FnOnce() -> Msg + Send + 'static>;

/// A wrapper for synchronous commands.
///
/// A [`SyncCmd`] is executed immediately (on the command worker) and produces a [`Msg`].
pub struct SyncCmd(pub CmdFn);

/// A wrapper for asynchronous commands.
///
/// In `matcha`, async commands are still represented as a [`CmdFn`]. The runtime will
/// call [`Model::execute`] for [`Cmd::Async`] and allow you to schedule further commands.
pub struct AsyncCmd(pub CmdFn);

/// A command produced by a model.
///
/// Commands represent side effects (I/O, timers, background work) and are executed
/// outside of the model update loop.
pub enum Cmd {
    /// Execute a command and send the returned message into the update loop.
    Sync(SyncCmd),
    /// Execute via [`Model::execute`].
    Async(AsyncCmd),
}

impl Cmd {
    /// Construct a synchronous command.
    pub fn sync(f: CmdFn) -> Self {
        Self::Sync(SyncCmd(f))
    }

    /// Construct an asynchronous command.
    pub fn r#async(f: CmdFn) -> Self {
        Self::Async(AsyncCmd(f))
    }
}

#[macro_export]
/// Create a [`Cmd::Sync`] command from an expression producing a [`Msg`].
///
/// This is a convenience macro for `Cmd::sync(Box::new(move || ...))`.
macro_rules! sync {
    ($expr:expr) => {
        Cmd::sync(Box::new(move || $expr))
    };
}

#[macro_export]
/// Create a [`Cmd::Async`] command from an expression producing a [`Msg`].
///
/// This is a convenience macro for `Cmd::r#async(Box::new(move || ...))`.
macro_rules! r#async {
    ($expr:expr) => {
        Cmd::r#async(Box::new(move || $expr))
    };
}

/// Program is a terminal user interface.
pub struct Program<M> {
    /// tea model
    model: M,
    /// Extensions
    extensions: Extensions,
    /// window size
    size: (u16, u16),
    /// if alt screen enabled, set `true`
    alt_screen: bool,
    /// terminal
    term: Box<dyn Termable>,
    /// optional external input channel (for tests/adapters)
    input_rx: Option<mpsc::Receiver<Msg>>,
}

/// batchMsg is the internal message used to perform a bunch of commands. You
/// can send a batchMsg with Batch.
pub type BatchMsg = Vec<Cmd>;

/// A built in command that combines multiple commands together.
///
/// These commands are executed in parallel, just like normal.
pub fn batch(msgs: BatchMsg) -> Cmd {
    Cmd::sync(Box::new(|| Box::new(msgs)))
}

/// EnterAltScreen is a special command that tells the Bubble Tea program to
/// enter the alternate screen buffer.
///
/// Because commands run asynchronously, this command should not be used in your
/// model's Init function. To initialize your program with the altscreen enabled
/// use the WithAltScreen ProgramOption instead.
pub fn enter_alt_screen() -> Msg {
    Box::new(EnterAltScreenMsg)
}

/// Create a command that sleeps for `d` and then emits the message returned by `f`.
///
/// This is a small helper for building timer-based behavior.
pub fn tick<F>(d: std::time::Duration, f: F) -> Cmd
where
    F: FnOnce() -> Msg + Send + 'static,
{
    Cmd::sync(Box::new(move || {
        std::thread::sleep(d);
        f()
    }))
}

/// A marker message type commonly used with [`tick`].
pub struct TickMsg;

/// enterAltScreenMsg in an internal message signals that the program should
/// enter alternate screen buffer. You can send a enterAltScreenMsg with
/// EnterAltScreen.
pub struct EnterAltScreenMsg;

/// ExitAltScreenMsg in an internal message signals that the program should exit
/// alternate screen buffer. You can send a exitAltScreenMsg with ExitAltScreen.
pub struct ExitAltScreenMsg;

/// NewProgram creates a new Program.
impl<M: Model> Program<M> {
    /// Create a new program using the default terminal backend.
    pub fn new(model: M, extensions: Extensions) -> Self {
        let term = DefaultTerminal;
        let (w, h) = term.size().unwrap();
        Self {
            model,
            extensions,
            size: (w, h),
            alt_screen: false,
            term: Box::new(term),
            input_rx: None,
        }
    }

    /// Create a new program using a custom terminal backend.
    ///
    /// This is useful for testing or integrating with non-standard terminals.
    pub fn new_with_terminal(model: M, extensions: Extensions, term: Box<dyn Termable>) -> Self {
        let (w, h) = term.size().unwrap();
        Self {
            model,
            extensions,
            size: (w, h),
            alt_screen: false,
            term,
            input_rx: None,
        }
    }

    /// Override event input stream with external message receiver.
    ///
    /// This is mainly intended for integration tests and terminal adapters.
    pub fn with_input_receiver(mut self, rx: mpsc::Receiver<Msg>) -> Self {
        self.input_rx = Some(rx);
        self
    }

    /// Enable alternate screen buffer from the start.
    ///
    /// This is the recommended mode for full-screen TUIs, and makes resize redraw far more stable.
    pub fn with_alt_screen(mut self) -> Self {
        self.alt_screen = true;
        self
    }

    /// Start the event loop and run until a quit message is received.
    pub async fn start(self) -> anyhow::Result<()> {
        self.inner_start().await?;
        Ok(())
    }

    async fn init(self, cmd_tx: Sender<Cmd>) -> Self {
        // Initialize the program.
        let inited = self.model.init(&InitInput { size: self.size });
        if let Some(cmd) = inited.1 {
            cmd_tx.send(cmd).await.unwrap();
        }
        Self {
            model: inited.0,
            ..self
        }
    }

    /// StartReturningModel initializes the program. Returns the final model.
    async fn inner_start(mut self) -> anyhow::Result<()> {
        // mpsc for message
        let (msg_tx, msg_rx) = mpsc::channel::<Msg>(100);

        // mpsc for command
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>(100);

        // Initialize the program.
        self = self.init(cmd_tx.clone()).await;

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let event_tx = msg_tx.clone();

        let input_handle = if let Some(mut input_rx) = self.input_rx.take() {
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        maybe_msg = input_rx.recv() => {
                            match maybe_msg {
                                Some(msg) => {
                                    if event_tx.send(msg).await.is_err() {
                                        return;
                                    }
                                }
                                None => return,
                            }
                        }
                        _ = (&mut shutdown_rx) => return,
                    }
                }
            })
        } else {
            let mut reader = EventStream::new();
            tokio::spawn(async move {
                loop {
                    let event = reader.next().fuse();

                    #[cfg(feature = "tracing")]
                    tracing::trace!("event {:?} recieved", &event);

                    tokio::select! {
                        maybe_event = event => {
                            let res = match maybe_event {
                                Some(Ok(Event::Key(event))) => event_tx.send(Box::new(event)).await,
                                Some(Ok(Event::Mouse(event))) => event_tx.send(Box::new(event)).await,
                                Some(Ok(Event::Resize(x, y))) => event_tx.send(Box::new(ResizeEvent(x, y))).await,
                                _ => Ok(()),
                            };
                            if res.is_err() {
                                #[cfg(feature = "tracing")]
                                tracing::error!("event {:?} recieved", res);
                                return;
                            }
                        },
                        _ = (&mut shutdown_rx) => {
                            // shutdown loop if oneshot emitted.
                            return;
                        }
                    }
                }
            })
        };

        // clone sender for executor
        let exec_tx = msg_tx.clone();
        let re_cmd_tx = cmd_tx.clone();

        let message_handle = tokio::spawn(async move {
            let mut rx = cmd_rx;
            loop {
                if let Some(cmd) = rx.recv().await {
                    let tx = exec_tx.clone();
                    let cmd_tx = re_cmd_tx.clone();
                    match cmd {
                        Cmd::Async(cmd) => {
                            let ext = self.extensions.clone();
                            tokio::spawn(async move {
                                let res = M::execute(ext, cmd).await;
                                match res {
                                    Some(Cmd::Sync(SyncCmd(cmd))) => {
                                        let msg = cmd();
                                        if let Err(e) = tx.send(msg).await {
                                            panic!("Failed to send message error. reason: {:?}", e);
                                        }
                                    }
                                    Some(cmd) => {
                                        if let Err(e) = cmd_tx.send(cmd).await {
                                            panic!("Failed to send message error. reason: {:?}", e);
                                        }
                                    }
                                    _ => {}
                                }
                            });
                        }
                        Cmd::Sync(SyncCmd(cmd)) => {
                            let msg = cmd();
                            if let Err(e) = tx.send(msg).await {
                                panic!("Failed to send message error. reason: {:?}", e);
                            }
                        }
                    }
                }
            }
        });

        // initial rendering
        self.term.hide_cursor()?;
        self.term.enable_raw_mode()?;
        let used_alt_screen = self.alt_screen;
        if used_alt_screen {
            self.term.enter_alt_screen()?;
            self.term.clear_all()?;
        }
        let run_result: anyhow::Result<()> = async {
            let mut prev_view = formatter::format(self.model.view(), self.size);
            self.term.print(&prev_view)?;

            // main loop
            let mut rx = msg_rx;
            loop {
                let msg = rx.recv().await.unwrap();

                #[cfg(feature = "tracing")]
                let span = tracing::info_span!("handle_message");
                #[cfg(feature = "tracing")]
                let _guard = span.enter();

                if msg.is::<QuitMsg>() {
                    break;
                }

                if msg.is::<BatchMsg>() {
                    if let Ok(batch) = msg.downcast::<BatchMsg>() {
                        for cmd in batch.into_iter() {
                            cmd_tx.send(cmd).await.unwrap();
                        }
                    }
                    continue;
                }

                if let Some(event) = msg.downcast_ref::<ResizeEvent>() {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("resize event recieved w = {}, h = {}", event.0, event.1);
                    self.size = (event.0, event.1);
                }

                if msg.is::<EnterAltScreenMsg>() {
                    self.alt_screen = true;
                    self.term.enter_alt_screen()?;
                    self.term.clear_all()?;
                }

                let (m, cmd) = self.model.update(&msg);
                self.model = m;

                if let Some(cmd) = cmd {
                    if cmd_tx.send(cmd).await.is_err() {
                        break;
                    }
                }

                let current_view = formatter::format(self.model.view(), self.size);

                #[cfg(feature = "tracing")]
                tracing::trace!("re-rendered");

                // Skip terminal clear/print when frame output is unchanged.
                if current_view == prev_view {
                    continue;
                }

                if self.alt_screen {
                    self.term.clear_all()?;
                } else {
                    self.term.move_to_column(0)?;
                    if prev_view.matches("\r\n").count() == 0 {
                        self.term.clear_current_line()?;
                    } else {
                        self.term.clear_current_line()?;
                        for _ in 0..prev_view.matches("\r\n").count() {
                            self.term.clear_current_line_and_move_previous()?;
                        }
                    }
                }

                self.term.print(&current_view)?;
                prev_view = current_view;
            }
            Ok(())
        }
        .await;

        #[cfg(feature = "tracing")]
        tracing::trace!("clean up program");

        message_handle.abort();
        let _ = shutdown_tx.send(true);
        input_handle.abort();

        let cleanup_result = Self::cleanup_terminal(self.term.as_ref(), used_alt_screen);
        run_result.and(cleanup_result)
    }

    fn cleanup_terminal(term: &dyn Termable, used_alt_screen: bool) -> anyhow::Result<()> {
        let mut first_error = None;
        let mut record = |result: Result<(), std::io::Error>, label: &str| {
            if let Err(error) = result {
                if first_error.is_none() {
                    first_error = Some(anyhow::anyhow!("failed to {}: {}", label, error));
                }
            }
        };

        // Prioritize raw-mode restoration so Ctrl+C works again.
        record(term.disable_raw_mode(), "disable raw mode");
        record(term.show_cursor(), "show cursor");
        record(term.disable_mouse_capture(), "disable mouse capture");
        if used_alt_screen {
            record(term.leave_alt_screen(), "leave alternate screen");
        }

        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(())
    }
}

/// Event representing a terminal resize (x, y).
/// Boxed as a message so it can be sent to the application.
pub struct ResizeEvent(pub u16, pub u16);

#[cfg(test)]
mod tests {
    use std::{
        fmt::Display,
        sync::{Arc, Mutex},
    };
    use tokio::sync::mpsc;

    use crate::{
        quit, Cmd, Extensions, KeyCode, KeyEvent, KeyModifiers, Model, Msg, Program, Termable,
    };

    struct FakeTerminal {
        printed: Arc<Mutex<Vec<String>>>,
    }

    impl FakeTerminal {
        fn new(printed: Arc<Mutex<Vec<String>>>) -> Self {
            Self { printed }
        }
    }

    impl Termable for FakeTerminal {
        fn size(&self) -> Result<(u16, u16), std::io::Error> {
            Ok((80, 24))
        }
        fn hide_cursor(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn show_cursor(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn enable_raw_mode(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn disable_raw_mode(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn print(&self, v: &str) -> Result<(), std::io::Error> {
            self.printed.lock().unwrap().push(v.to_string());
            Ok(())
        }
        fn enter_alt_screen(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn leave_alt_screen(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn enable_mouse_capture(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn disable_mouse_capture(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn move_to_column(&self, _y: u16) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn move_to(&self, _x: u16, _y: u16) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn cursor_position(&self) -> Result<(u16, u16), std::io::Error> {
            Ok((0, 0))
        }
        fn clear_all(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn clear_current_line(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        fn clear_current_line_and_move_previous(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    struct TestModel {
        seen: String,
    }

    impl Model for TestModel {
        fn update(mut self, msg: &Msg) -> (Self, Option<Cmd>) {
            if let Some(key) = msg.downcast_ref::<KeyEvent>() {
                if let KeyCode::Char(ch) = key.code {
                    self.seen.push(ch);
                    if ch == 'q' {
                        return (self, Some(Cmd::sync(Box::new(quit))));
                    }
                }
            }
            (self, None)
        }

        fn view(&self) -> impl Display {
            self.seen.clone()
        }
    }

    #[tokio::test]
    async fn program_can_run_with_external_input_receiver() {
        let printed = Arc::new(Mutex::new(Vec::<String>::new()));
        let term = FakeTerminal::new(printed.clone());
        let (tx, rx) = mpsc::channel::<Msg>(8);

        tx.send(Box::new(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        )))
        .await
        .unwrap();
        tx.send(Box::new(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )))
        .await
        .unwrap();
        drop(tx);

        let p = Program::new_with_terminal(
            TestModel {
                seen: String::new(),
            },
            Extensions::default(),
            Box::new(term),
        )
        .with_input_receiver(rx);
        p.start().await.unwrap();

        let out = printed.lock().unwrap();
        assert!(!out.is_empty(), "program should render at least once");
    }

    struct FailingCleanupTerminal {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl FailingCleanupTerminal {
        fn new(calls: Arc<Mutex<Vec<String>>>) -> Self {
            Self { calls }
        }

        fn record_call(&self, name: &str) {
            self.calls.lock().unwrap().push(name.to_string());
        }
    }

    impl Termable for FailingCleanupTerminal {
        fn size(&self) -> Result<(u16, u16), std::io::Error> {
            Ok((80, 24))
        }

        fn hide_cursor(&self) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn show_cursor(&self) -> Result<(), std::io::Error> {
            self.record_call("show_cursor");
            Err(std::io::Error::other("show cursor failed"))
        }

        fn enable_raw_mode(&self) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn disable_raw_mode(&self) -> Result<(), std::io::Error> {
            self.record_call("disable_raw_mode");
            Ok(())
        }

        fn print(&self, _v: &str) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn enter_alt_screen(&self) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn leave_alt_screen(&self) -> Result<(), std::io::Error> {
            self.record_call("leave_alt_screen");
            Ok(())
        }

        fn enable_mouse_capture(&self) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn disable_mouse_capture(&self) -> Result<(), std::io::Error> {
            self.record_call("disable_mouse_capture");
            Ok(())
        }

        fn move_to_column(&self, _y: u16) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn move_to(&self, _x: u16, _y: u16) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn cursor_position(&self) -> Result<(u16, u16), std::io::Error> {
            Ok((0, 0))
        }

        fn clear_all(&self) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn clear_current_line(&self) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn clear_current_line_and_move_previous(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[test]
    fn cleanup_terminal_attempts_raw_mode_restore_even_if_other_steps_fail() {
        let calls = Arc::new(Mutex::new(vec![]));
        let term = FailingCleanupTerminal::new(calls.clone());

        let result = Program::<TestModel>::cleanup_terminal(&term, true);

        assert!(
            result.is_err(),
            "cleanup should report first encountered error"
        );
        let calls = calls.lock().unwrap();
        assert_eq!(
            calls.first().map(String::as_str),
            Some("disable_raw_mode"),
            "raw mode should be restored first"
        );
        assert!(
            calls.iter().any(|call| call == "disable_mouse_capture"),
            "cleanup should continue after failures"
        );
        assert!(
            calls.iter().any(|call| call == "leave_alt_screen"),
            "alt-screen cleanup should still be attempted"
        );
    }
}
