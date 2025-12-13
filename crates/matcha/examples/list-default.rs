use std::fmt::Display;

use chagashi::list::{DefaultItemDelegate, Item, Model as ListModel};
use matcha::{Cmd, InitInput, KeyEvent, Model, Msg};

#[derive(Clone)]
struct ListItem {
    title: String,
}

impl Item for ListItem {
    fn filter_value(&self) -> String {
        self.title.clone()
    }
}

struct App {
    list: ListModel,
}

impl Default for App {
    fn default() -> Self {
        let items = vec![
            Box::new(ListItem {
                title: "Raspberry Pi's".to_string(),
            }) as Box<dyn Item>,
            Box::new(ListItem {
                title: "Nutella".to_string(),
            }) as Box<dyn Item>,
            Box::new(ListItem {
                title: "Bitter melon".to_string(),
            }) as Box<dyn Item>,
            Box::new(ListItem {
                title: "Nice socks".to_string(),
            }) as Box<dyn Item>,
            Box::new(ListItem {
                title: "Eight hours of sleep".to_string(),
            }) as Box<dyn Item>,
            Box::new(ListItem {
                title: "Linux".to_string(),
            }) as Box<dyn Item>,
            Box::new(ListItem {
                title: "Pottery".to_string(),
            }) as Box<dyn Item>,
        ];

        let mut list = ListModel::new()
            .with_delegate(DefaultItemDelegate)
            .with_items(items);
        list.set_title("My Fave Things");

        Self { list }
    }
}

impl Model for App {
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        let (list, cmd) = self.list.init(input);
        (Self { list }, cmd)
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            if key_event.modifiers == matcha::KeyModifiers::CONTROL {
                if let matcha::KeyCode::Char('c') = key_event.code {
                    return (self, Some(matcha::sync!(Box::new(()) as Msg)));
                }
            }
        }

        let (list, cmd) = self.list.update(msg);
        (Self { list }, cmd)
    }

    fn view(&self) -> impl Display {
        self.list.view()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::default();
    let extensions = matcha::Extensions::default();
    let program = matcha::Program::new(app, extensions);
    program.start().await?;
    Ok(())
}
