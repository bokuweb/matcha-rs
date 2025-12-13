use std::fmt::Display;

use chagashi::textinput::TextInput;
use matcha::{quit, Cmd, Extensions, InitInput, KeyCode, KeyEvent, Model, Msg, Program};

struct App {
    input: TextInput,
}

impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        let (input, cmd) = self.input.focus();
        (Self { input }, cmd)
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(msg) = msg.downcast_ref::<KeyEvent>() {
            if msg.code == KeyCode::Esc {
                return (self, Some(matcha::sync!(quit())));
            }
        }

        let (input, cmd) = self.input.update(msg);
        (Self { input }, cmd)
    }

    fn view(&self) -> impl Display {
        "What’s your favorite language\n".to_string()
            + "\n"
            + &format!("{}", self.input.view())
            + "\n"
            + "\n"
            + "(esc to quit)"
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let input = TextInput::new().set_placeholder("Rust");
    let p = Program::new(App { input }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
