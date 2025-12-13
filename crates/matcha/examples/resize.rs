use std::fmt::Display;

use matcha::{quit, Cmd, Extensions, Model, Msg, Program, ResizeEvent};
use matcha::{KeyCode, KeyEvent, KeyModifiers};

struct App {
    w: u16,
    h: u16,
}

impl Model for App {
    fn init(self, input: &matcha::InitInput) -> (Self, Option<Cmd>) {
        (
            Self {
                w: input.size.0,
                h: input.size.1,
            },
            None,
        )
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(event) = msg.downcast_ref::<ResizeEvent>() {
            return (
                Self {
                    w: event.0,
                    h: event.1,
                },
                None,
            );
        }

        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            if let KeyModifiers::CONTROL = key_event.modifiers {
                match key_event.code {
                    KeyCode::Char('c') => return (self, Some(matcha::sync!(quit()))),
                    _ => return (self, None),
                }
            }
        }
        (self, None)
    }

    fn view(&self) -> impl Display {
        format!("width = {}, height = {}", self.w, self.h)
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(App { w: 0, h: 0 }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
