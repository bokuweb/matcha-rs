use std::fmt::Display;

use chagashi::{tabs::Tab, tabs::Tabs};
use matcha::boxed;
use matcha::{quit, Cmd, Extensions, KeyEvent, Model, Msg, Program};

#[derive(Clone)]
struct Static(&'static str);

impl Model for Static {
    fn view(&self) -> impl Display {
        self.0.to_string()
    }
}

struct App {
    tabs: Tabs,
}

impl Default for App {
    fn default() -> Self {
        let tabs = vec![
            Tab::new(
                "Sushi",
                boxed(Static(
                    "Sushi\n- Vinegared rice with fresh toppings\n- Popular styles: nigiri, maki, chirashi\n- Tip: try seasonal fish at a local counter",
                )),
            ),
            Tab::new(
                "Ramen",
                boxed(Static(
                    "Ramen\n- Noodles in a rich broth\n- Common bases: shoyu, miso, tonkotsu, shio\n- Toppings: chashu, egg, nori, scallions",
                )),
            ),
            Tab::new(
                "Tempura",
                boxed(Static(
                    "Tempura\n- Lightly battered, deep-fried seafood and vegetables\n- Best served hot and crisp\n- Dip: tentsuyu or a pinch of salt",
                )),
            ),
            Tab::new(
                "Okonomiyaki",
                boxed(Static(
                    "Okonomiyaki\n- Savory pancake with cabbage and your choice of fillings\n- Topped with sauce, mayo, aonori, and bonito flakes\n- Regional favorites: Osaka style, Hiroshima style",
                )),
            ),
            Tab::new(
                "Matcha",
                boxed(Static(
                    "Matcha\n- Stone-ground green tea powder\n- Flavor: grassy, umami, slightly bitter\n- Great in: tea, latte, and wagashi sweets",
                )),
            ),
        ];
        Self {
            tabs: Tabs::new(tabs),
        }
    }
}

impl Model for App {
    fn init(self, input: &matcha::InitInput) -> (Self, Option<Cmd>) {
        let (tabs, cmd) = self.tabs.init(input);
        (Self { tabs }, cmd)
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            if matcha::Key::from(key_event).matches(matcha::key!(ctrl - c)) {
                return (self, Some(matcha::sync!(quit())));
            }
            if matcha::Key::from(key_event).matches(matcha::key!(q)) {
                return (self, Some(matcha::sync!(quit())));
            }
        }

        let (tabs, cmd) = self.tabs.update(msg);
        (Self { tabs }, cmd)
    }

    fn view(&self) -> impl Display {
        self.tabs.view()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::default();
    let program = Program::new(app, Extensions::default()).with_alt_screen();
    program.start().await?;
    Ok(())
}
