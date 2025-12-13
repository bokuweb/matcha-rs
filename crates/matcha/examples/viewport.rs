use std::fmt::Display;

use chagashi::viewport::{Viewport, ViewportOption};
use matcha::{quit, Cmd, Extensions, KeyEvent, Model, Msg, Program};

struct App {
    viewport: Viewport<Child>,
}

impl Model for App {
    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            if matcha::Key::from(key_event).matches(matcha::key!(ctrl - c)) {
                return (self, Some(matcha::sync!(quit())));
            }
        };

        let (viewport, cmd) = self.viewport.update(msg);
        (Self { viewport }, cmd)
    }

    fn view(&self) -> impl Display {
        self.viewport.view()
    }
}

struct Child;

impl Model for Child {
    fn view(&self) -> impl Display {
        let lines = [
            "The quick brown fox jumps over the lazy dog.",
            "Innovation distinguishes between a leader and a follower.",
            "To be or not to be, that is the question.",
            "A journey of a thousand miles begins with a single step.",
            "All our dreams can come true, if we have the courage to pursue them.",
            "The only way to do great work is to love what you do.",
            "Life is what happens when you're busy making other plans.",
            "The best and most beautiful things in the world cannot be seen or even touched - they must be felt with the heart.",
            "It does not matter how slowly you go as long as you do not stop.",
            "Success is not final, failure is not fatal: It is the courage to continue that counts.",
            "You must be the change you wish to see in the world.",
            "I think, therefore I am.",
            "The only thing we have to fear is fear itself.",
            "Ask not what your country can do for you – ask what you can do for your country.",
            "In the end, it's not the years in your life that count. It's the life in your years.",
            "Life is either a daring adventure or nothing at all.",
            "You miss 100% of the shots you don't take.",
            "Whether you think you can or you think you can't, you're right.",
            "The purpose of our lives is to be happy.",
            "Everything you can imagine is real.",
        ];
        lines.join("\n")
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(
        App {
            viewport: Viewport::new(
                Child,
                (80, 12),
                ViewportOption {
                    selection: true,
                    ..Default::default()
                },
            ),
        },
        Extensions::default(),
    );
    p.start().await.unwrap();
    Ok(())
}
