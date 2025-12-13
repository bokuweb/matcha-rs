use std::fmt::Display;

use chagashi::Flex;
use matcha::boxed;
use matcha::{key, Cmd, Extensions, Key, KeyEvent, Model, Msg, Program};

#[derive(Clone)]
struct Item(&'static str);

impl Model for Item {
    fn view(&self) -> impl Display {
        // Make the output intentionally wide so we can see column widths via clamp/padding.
        let bar = "─".repeat(200);
        format!("{}\n{}\n{}", self.0, bar, bar)
    }
}

struct App {
    flex: Flex,
}

impl App {
    fn new() -> Self {
        let children = vec![
            boxed(Item("Item 1")),
            boxed(Item("Item 2")),
            boxed(Item("Item 3")),
            boxed(Item("Item 4")),
        ];
        // Tune `min_item_width` so columns drop (e.g. 4→2→1) as the terminal gets narrower.
        let flex = Flex::new(children).gap(2).min_item_width(20).wrap(true);
        Self { flex }
    }
}

impl Model for App {
    fn init(self, input: &matcha::InitInput) -> (Self, Option<Cmd>) {
        let (flex, cmd) = self.flex.init(input);
        (Self { flex }, cmd)
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            let k = Key::from(key_event);
            if k.matches(key!(q)) || k.matches(key!(ctrl - c)) {
                return (self, Some(matcha::sync!(matcha::quit())));
            }
        }

        let (flex, cmd) = self.flex.update(msg);
        (Self { flex }, cmd)
    }

    fn view(&self) -> impl Display {
        self.flex.view().to_string()
    }
}

#[tokio::main]
async fn main() {
    // Flex example is intended to be used in alt-screen mode for stable redraw on resize.
    let p = Program::new(App::new(), Extensions::default()).with_alt_screen();
    p.start().await.unwrap();
}
