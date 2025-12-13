mod extension;
mod formatter;
mod key;
mod messages;
mod termable;
mod terminal;

pub use extension::*;
pub use formatter::*;
pub use key::*;
pub use messages::*;
use termable::Termable;
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
pub struct InitInput {
    pub size: (u16, u16),
}

#[async_trait::async_trait]
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

    /// An asynchronous function that can execute commands received from either 'init' or 'update'.
    /// This function needs to accept a 'Cmd' and return an 'Option<Msg>'.
    /// If not needed, implementation is not required.
    /// # Example
    ///
    /// ```ignore
    /// async fn execute(AsyncCmd(cmd): AsyncCmd) -> Option<Cmd> {
    ///     let msg = cmd();
    ///     if msg.downcast_ref::<AsyncMsg>().is_some() {
    ///         tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    ///         return Some(Cmd::sync(Box::new(DoneMsg)));
    ///     }
    ///     None
    /// }
    /// ```
    ///
    async fn execute(_ext: Extensions, _cmd: AsyncCmd) -> Option<Cmd> {
        None
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

pub struct SyncCmd(pub CmdFn);

pub struct AsyncCmd(pub CmdFn);

pub enum Cmd {
    Sync(SyncCmd),
    Async(AsyncCmd),
}

impl Cmd {
    pub fn sync(f: CmdFn) -> Self {
        Self::Sync(SyncCmd(f))
    }

    pub fn r#async(f: CmdFn) -> Self {
        Self::Async(AsyncCmd(f))
    }
}

#[macro_export]
macro_rules! sync {
    ($expr:expr) => {
        Cmd::sync(Box::new(move || $expr))
    };
}

#[macro_export]
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

pub fn tick<F>(d: std::time::Duration, f: F) -> Cmd
where
    F: FnOnce() -> Msg + Send + 'static,
{
    Cmd::sync(Box::new(move || {
        std::thread::sleep(d);
        f()
    }))
}

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
    pub fn new(model: M, extensions: Extensions) -> Self {
        let term = DefaultTerminal;
        let (w, h) = term.size().unwrap();
        Self {
            model,
            extensions,
            size: (w, h),
            alt_screen: false,
            term: Box::new(term),
        }
    }

    pub fn new_with_terminal(model: M, extensions: Extensions, term: Box<dyn Termable>) -> Self {
        let (w, h) = term.size().unwrap();
        Self {
            model,
            extensions,
            size: (w, h),
            alt_screen: false,
            term,
        }
    }

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

        let mut reader = EventStream::new();

        let event_tx = msg_tx.clone();

        let input_handle = tokio::spawn(async move {
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
        });

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

        #[cfg(feature = "tracing")]
        tracing::trace!("clean up program");

        message_handle.abort();
        shutdown_tx.send(true).unwrap();
        input_handle.abort();

        self.term.show_cursor()?;
        self.term.disable_mouse_capture()?;

        if self.alt_screen {
            self.term.leave_alt_screen()?;
        }

        self.term.disable_raw_mode()?;

        Ok(())
    }
}

/// Event representing a terminal resize (x, y).
/// Boxed as a message so it can be sent to the application.
pub struct ResizeEvent(pub u16, pub u16);
