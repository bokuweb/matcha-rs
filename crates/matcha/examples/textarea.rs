use std::fmt::Display;

use chagashi::textarea::Textarea;
use matcha::{batch, quit, Cmd, Extensions, InitInput, KeyCode, KeyEvent, Model, Msg, Program};

struct App {
    textarea: Textarea,
}

impl Model for App {
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        let (borderize, init_cmd) = self.textarea.init(&InitInput {
            size: (60, std::cmp::min(8, input.size.1)),
        });
        let (textarea, focus_cmd) = borderize.focus();
        let mut cmds: matcha::BatchMsg = vec![];
        init_cmd.into_iter().for_each(|c| cmds.push(c));
        focus_cmd.into_iter().for_each(|c| cmds.push(c));
        (Self { textarea }, Some(batch(cmds)))
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(msg) = msg.downcast_ref::<KeyEvent>() {
            if msg.code == KeyCode::Esc {
                return (self, Some(matcha::sync!(quit())));
            }
        }
        let (textarea, cmd) = self.textarea.update(msg);
        (Self { textarea }, cmd)
    }

    fn view(&self) -> impl Display {
        self.textarea.view().to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let textarea = Textarea::with_content("Hello\nWorld!").border();
    let p = Program::new(App { textarea }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
