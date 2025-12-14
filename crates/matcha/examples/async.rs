use std::fmt::Display;

use matcha::{
    quit, style, AsyncCmd, Cmd, Extensions, InitInput, KeyEvent, Model, Msg, Program, Stylize,
};

pub fn init() -> Msg {
    Box::new(AsyncMsg) as Msg
}

pub fn done() -> Msg {
    Box::new(DoneMsg) as Msg
}

pub struct AsyncMsg;

pub struct DoneMsg;

struct App {
    done: bool,
}

#[async_trait::async_trait]
impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        (self, Some(matcha::r#async!(init())))
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if msg.downcast_ref::<KeyEvent>().is_some() {
            return (self, Some(matcha::sync!(quit())));
        }
        if msg.downcast_ref::<DoneMsg>().is_some() {
            return (Self { done: true }, None);
        }
        (self, None)
    }

    fn view(&self) -> impl Display {
        if self.done {
            style("Completed.").negative().to_string()
        } else {
            style("Waiting for the completion of an async task.")
                .negative()
                .to_string()
        }
    }

    async fn execute(_ext: Extensions, AsyncCmd(cmd): AsyncCmd) -> Option<Cmd> {
        let msg = cmd();
        if msg.downcast_ref::<AsyncMsg>().is_some() {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            return Some(matcha::sync!(done()));
        }
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(App { done: false }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
