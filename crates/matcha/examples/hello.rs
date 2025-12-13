use std::fmt::Display;

use matcha::{quit, Cmd, Extensions, KeyEvent, Model, Msg, Program};

struct App;

impl Model for App {
    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            if matcha::Key::from(key_event).matches(matcha::key!(ctrl - c)) {
                return (self, Some(matcha::sync!(quit())));
            };
            return (self, None);
        };
        (self, None)
    }

    fn view(&self) -> impl Display {
        "Hello World"
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(App, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
