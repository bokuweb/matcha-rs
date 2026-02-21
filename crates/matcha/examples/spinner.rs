use std::fmt::Display;

use chagashi::spinner::SpinnerType;
use matcha::{
    quit, style, Cmd, Color, Extensions, InitInput, KeyCode, KeyEvent, Model, Msg, Program,
};

pub struct TickMsg;

struct App {
    spinner: chagashi::spinner::Spinner,
    quitting: bool,
}

impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        let cmd = self.spinner.tick(0);
        (self, Some(cmd))
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(e) = msg.downcast_ref::<KeyEvent>() {
            if e.modifiers.is_empty() && e.code == KeyCode::Char('c') {
                let color = self.spinner.color();
                let t = &self.spinner.spinner_type();
                let n = match t {
                    SpinnerType::Line { .. } => SpinnerType::dot(),
                    SpinnerType::Dot { .. } => SpinnerType::mini_dot(),
                    SpinnerType::MiniDot { .. } => SpinnerType::jump(),
                    SpinnerType::Jump { .. } => SpinnerType::pulse(),
                    SpinnerType::Pulse { .. } => SpinnerType::points(),
                    SpinnerType::Points { .. } => SpinnerType::globe(),
                    SpinnerType::Globe { .. } => SpinnerType::moon(),
                    SpinnerType::Moon { .. } => SpinnerType::monkey(),
                    SpinnerType::Monkey { .. } => SpinnerType::meter(),
                    SpinnerType::Meter { .. } => SpinnerType::hamburger(),
                    SpinnerType::Hamburger { .. } => SpinnerType::line(),
                };

                let s = if let Some(color) = color {
                    chagashi::spinner::Spinner::new(n).set_color(color)
                } else {
                    chagashi::spinner::Spinner::new(n)
                };
                let cmd = Some(s.tick(0));
                return (
                    Self {
                        spinner: s,
                        quitting: self.quitting,
                    },
                    cmd,
                );
            }

            return (
                Self {
                    quitting: true,
                    ..self
                },
                Some(matcha::sync!(quit())),
            );
        }
        let (s, c) = self.spinner.update(msg);
        (Self { spinner: s, ..self }, c)
    }

    fn view(&self) -> impl Display {
        let v = style(format!(
            "\n\n   {} Loading forever...press `c` to change spinner type, press other key to quit\n\n",
            self.spinner.view()
        ))
        .to_string();

        if self.quitting {
            v + "\n"
        } else {
            v
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let t = SpinnerType::dot();
    let p = Program::new(
        App {
            spinner: chagashi::spinner::Spinner::new(t).set_color(Color::AnsiValue(205)),
            quitting: false,
        },
        Extensions::default(),
    );
    p.start().await.unwrap();
    Ok(())
}
