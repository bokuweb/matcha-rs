use std::fmt::Display;

use matcha::{quit, style, Cmd, Extensions, InitInput, KeyEvent, Model, Msg, Program, Stylize};

pub fn tick() -> Msg {
    std::thread::sleep(std::time::Duration::from_millis(1000));
    Box::new(TickMsg) as Msg
}

pub struct TickMsg;

struct App {
    count: usize,
}

#[async_trait::async_trait]
impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        (self, Some(matcha::sync!(tick())))
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if msg.downcast_ref::<KeyEvent>().is_some() {
            return (self, Some(matcha::sync!(quit())));
        }
        if msg.downcast_ref::<TickMsg>().is_some() {
            let count = self.count - 1;
            if count == 0 {
                return (self, Some(matcha::sync!(quit())));
            }
            return (Self { count }, Some(matcha::sync!(tick())));
        };
        (self, None)
    }

    fn view(&self) -> impl Display {
        style(format!(
            "Hi. This program will exit in {} seconds. To quit sooner press any key.\n",
            self.count
        ))
        .negative()
        .to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(App { count: 5 }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
