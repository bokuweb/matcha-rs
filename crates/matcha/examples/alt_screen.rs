use std::fmt::Display;

use matcha::{
    enter_alt_screen, quit, style, Cmd, Extensions, InitInput, KeyEvent, Model, Msg, Program,
    Stylize,
};

struct App;

impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        (self, Some(matcha::sync!(enter_alt_screen())))
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if msg.downcast_ref::<KeyEvent>().is_some() {
            return (self, Some(matcha::sync!(quit())));
        }
        (self, None)
    }

    fn view(&self) -> impl Display {
        style("press any key to quit this mode.\n")
            .negative()
            .to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(App, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
