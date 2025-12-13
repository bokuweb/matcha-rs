use std::fmt::Display;

use chagashi::borderize::Borderize;
use matcha::{quit, Cmd, Extensions, KeyEvent, Model, Msg, Program};

struct Content;

impl Model for Content {
    fn view(&self) -> impl Display {
        "Hello\nWorld!!!!!!!!!!!!"
    }
}

struct App<M> {
    child: M,
}

impl<M: Model> Model for App<M> {
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
        self.child.view()
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(
        App {
            child: Borderize::new(Content),
        },
        Extensions::default(),
    );
    p.start().await.unwrap();
    Ok(())
}
