use std::fmt::{Display, Write};
use std::sync::Arc;

use crate::spinner::TickMsg;
use matcha::KeyCode;
use matcha::KeyEvent;
use matcha::{style, Cmd, Color as MatchaColor, InitInput, Model as MModel, Msg, Stylize};

// Define a matcha-compatible event type for easier use within this crate.
#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Other(Box<dyn std::any::Any + Send>),
}

impl Clone for Event {
    fn clone(&self) -> Self {
        match self {
            Event::Key(key) => Event::Key(*key),
            Event::Other(_) => Event::Other(Box::new(()) as Box<dyn std::any::Any + Send>),
        }
    }
}

use crate::spinner::{Spinner, SpinnerType};

/// Item is a trait that must be implemented by items that appear in the list.
pub trait Item: Send + Sync {
    /// FilterValue is the value we use when filtering against this item.
    fn filter_value(&self) -> String;
}

/// ItemDelegate encapsulates the functionality for all list items.
pub trait ItemDelegate: Send + Sync {
    /// Render renders the item's view.
    fn render(&self, w: &mut dyn Write, model: &Model, index: usize, item: &dyn Item);

    /// Height is the height of the list item.
    fn height(&self) -> usize;

    /// Spacing is the size of the horizontal gap between list items in cells.
    fn spacing(&self) -> usize;

    /// Update is the update loop for items.
    fn update(&self, event: Event, model: &mut Model) -> Option<Event>;
}

/// A small helper type to make styling ergonomics easier in this crate.
pub struct StylizeWrapper {
    pub content: String,
    pub fg_color: Option<MatchaColor>,
    pub bg_color: Option<MatchaColor>,
    pub bold: bool,
}

impl StylizeWrapper {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            fg_color: None,
            bg_color: None,
            bold: false,
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn bg(mut self, color: MatchaColor) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn with(mut self, color: MatchaColor) -> Self {
        self.fg_color = Some(color);
        self
    }
}

impl Clone for StylizeWrapper {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            fg_color: self.fg_color,
            bg_color: self.bg_color,
            bold: self.bold,
        }
    }
}

impl Stylize for StylizeWrapper {
    type Styled = matcha::crossterm::style::StyledContent<String>;

    fn stylize(self) -> Self::Styled {
        let mut styled = style(self.content.clone());
        if self.bold {
            styled = styled.bold();
        }
        if let Some(color) = self.fg_color {
            styled = styled.with(color);
        }
        if let Some(color) = self.bg_color {
            styled = styled.on(color);
        }
        styled
    }
}

/// Model contains the state for the list component
pub struct Model {
    // Display options
    show_title: bool,
    show_status_bar: bool,
    show_pagination: bool,
    show_help: bool,

    // Customization
    item_name_singular: String,
    item_name_plural: String,
    title: String,

    // Styling attributes
    title_style: StylizeWrapper,
    status_bar_style: StylizeWrapper,
    selected_item_style: StylizeWrapper,
    normal_item_style: StylizeWrapper,

    // State
    width: usize,
    height: usize,
    cursor: usize,
    per_page: usize,
    page: usize,
    total_pages: usize,

    // Items
    items: Vec<Arc<dyn Item>>,

    // Spinner
    spinner: Spinner,
    show_spinner: bool,

    // Status message
    status_message: String,

    // Delegate
    delegate: Box<dyn ItemDelegate>,

    // This flag determines whether the list should loop around when navigating
    // beyond the last or first item
    infinite_scrolling: bool,
}

impl Default for Model {
    fn default() -> Self {
        let spinner = Spinner::new(SpinnerType::line());

        Self {
            show_title: true,
            show_status_bar: true,
            show_pagination: true,
            show_help: true,

            item_name_singular: "item".to_string(),
            item_name_plural: "items".to_string(),
            title: "List".to_string(),

            title_style: StylizeWrapper::new("").bold(),
            status_bar_style: StylizeWrapper::new(""),
            selected_item_style: StylizeWrapper::new("").bg(MatchaColor::Blue),
            normal_item_style: StylizeWrapper::new(""),

            width: 80,
            height: 24,
            cursor: 0,
            per_page: 10,
            page: 0,
            total_pages: 1,

            items: Vec::new(),

            spinner,
            show_spinner: false,

            status_message: String::new(),

            delegate: Box::new(DefaultItemDelegate),
            infinite_scrolling: false,
        }
    }
}

#[derive(Clone)]
pub struct DefaultItemDelegate;

impl ItemDelegate for DefaultItemDelegate {
    fn render(&self, w: &mut dyn Write, model: &Model, index: usize, item: &dyn Item) {
        let mut style = if index == model.index() {
            model.selected_item_style.clone()
        } else {
            model.normal_item_style.clone()
        };
        style.content = item.filter_value();

        let _ = write!(w, "{}", style.stylize());
    }

    fn height(&self) -> usize {
        1
    }

    fn spacing(&self) -> usize {
        0
    }

    fn update(&self, _event: Event, _model: &mut Model) -> Option<Event> {
        None
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_delegate(mut self, delegate: impl ItemDelegate + Clone + 'static) -> Self {
        self.delegate = Box::new(delegate);
        self.update_pagination();
        self
    }

    pub fn with_items(mut self, items: Vec<Box<dyn Item>>) -> Self {
        self.items = items.into_iter().map(Arc::from).collect();
        self.update_pagination();
        self
    }

    pub fn set_items(&mut self, items: Vec<Box<dyn Item>>) {
        self.items = items.into_iter().map(Arc::from).collect();
        self.update_pagination();
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.update_pagination();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_show_title(&mut self, show: bool) {
        self.show_title = show;
        self.update_pagination();
    }

    pub fn set_show_status_bar(&mut self, show: bool) {
        self.show_status_bar = show;
        self.update_pagination();
    }

    pub fn set_show_pagination(&mut self, show: bool) {
        self.show_pagination = show;
        self.update_pagination();
    }

    pub fn set_show_help(&mut self, show: bool) {
        self.show_help = show;
        self.update_pagination();
    }

    pub fn set_status_bar_item_name(
        &mut self,
        singular: impl Into<String>,
        plural: impl Into<String>,
    ) {
        self.item_name_singular = singular.into();
        self.item_name_plural = plural.into();
    }

    pub fn start_spinner(&mut self) -> Option<Cmd> {
        self.show_spinner = true;
        let tag = self.spinner.id() + 1;
        Some(self.spinner.tick(tag))
    }

    pub fn stop_spinner(&mut self) {
        self.show_spinner = false;
    }

    pub fn toggle_spinner(&mut self) -> Option<Cmd> {
        if !self.show_spinner {
            self.start_spinner()
        } else {
            self.stop_spinner();
            None
        }
    }

    pub fn visible_items(&self) -> Vec<Arc<dyn Item>> {
        self.items.clone()
    }

    pub fn selected_item(&self) -> Option<Arc<dyn Item>> {
        let i = self.index();
        let items = self.visible_items();

        if i >= items.len() {
            return None;
        }

        Some(items[i].clone())
    }

    pub fn index(&self) -> usize {
        self.page * self.per_page + self.cursor
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            return;
        }

        // We're at the top of the page
        if self.page > 0 {
            self.page -= 1;
            self.cursor = self.per_page - 1;
        } else if self.infinite_scrolling {
            // Go to the last page
            self.page = self.total_pages - 1;
            let items_on_page = self.items_on_page();
            self.cursor = if items_on_page > 0 {
                items_on_page - 1
            } else {
                0
            };
        }
    }

    pub fn cursor_down(&mut self) {
        let items_on_page = self.items_on_page();

        if self.cursor + 1 < items_on_page {
            self.cursor += 1;
            return;
        }

        // We're at the bottom of the page
        if self.page + 1 < self.total_pages {
            self.page += 1;
            self.cursor = 0;
        } else if self.infinite_scrolling {
            // Go to the first page
            self.page = 0;
            self.cursor = 0;
        }
    }

    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
            // Make sure cursor is within bounds
            let items_on_page = self.items_on_page();
            if self.cursor >= items_on_page && items_on_page > 0 {
                self.cursor = items_on_page - 1;
            }
        }
    }

    pub fn next_page(&mut self) {
        if self.page + 1 < self.total_pages {
            self.page += 1;
            // Make sure cursor is within bounds
            let items_on_page = self.items_on_page();
            if self.cursor >= items_on_page && items_on_page > 0 {
                self.cursor = items_on_page - 1;
            }
        }
    }

    pub fn go_to_start(&mut self) {
        self.page = 0;
        self.cursor = 0;
    }

    pub fn go_to_end(&mut self) {
        self.page = self.total_pages - 1;
        let items_on_page = self.items_on_page();
        self.cursor = if items_on_page > 0 {
            items_on_page - 1
        } else {
            0
        };
    }

    fn update_pagination(&mut self) {
        let mut available_height = self.height;

        // Adjust for title
        if self.show_title {
            available_height = available_height.saturating_sub(1);
        }

        // Adjust for status bar
        if self.show_status_bar {
            available_height = available_height.saturating_sub(1);
        }

        // Adjust for pagination
        if self.show_pagination {
            available_height = available_height.saturating_sub(1);
        }

        // Adjust for help
        if self.show_help {
            available_height = available_height.saturating_sub(1);
        }

        // Calculate per_page
        let item_height = self.delegate.height() + self.delegate.spacing();
        self.per_page = if item_height > 0 {
            std::cmp::max(1, available_height / item_height)
        } else {
            1
        };

        // Calculate total_pages
        let total_items = self.visible_items().len();
        self.total_pages = if total_items == 0 {
            1
        } else {
            total_items.div_ceil(self.per_page)
        };

        // Ensure page is in bounds
        self.page = std::cmp::min(self.page, self.total_pages.saturating_sub(1));

        // Ensure cursor is in bounds
        let items_on_page = self.items_on_page();
        if self.cursor >= items_on_page && items_on_page > 0 {
            self.cursor = items_on_page - 1;
        }
    }

    fn items_on_page(&self) -> usize {
        let total_items = self.visible_items().len();
        if total_items == 0 {
            return 0;
        }

        let remaining = total_items - (self.page * self.per_page);
        std::cmp::min(remaining, self.per_page)
    }

    fn handle_key_event(&mut self, key: &KeyEvent) -> Option<Cmd> {
        match key.code {
            KeyCode::Up => {
                self.cursor_up();
            }
            KeyCode::Down => {
                self.cursor_down();
            }
            KeyCode::PageUp => {
                self.prev_page();
            }
            KeyCode::PageDown => {
                self.next_page();
            }
            KeyCode::Home => {
                self.go_to_start();
            }
            KeyCode::End => {
                self.go_to_end();
            }
            _ => {
                // Let the delegate handle the event
                // let mut model_clone = self.clone();
                // if let Some(event) = self
                //     .delegate
                //     .update(Event::Key(key.clone()), &mut model_clone)
                // {
                //     // Apply changes if needed
                //     *self = model_clone;
                //     return Some(Cmd::sync(Box::new(move || {
                //         Box::new(event) as Box<dyn std::any::Any + Send>
                //     })));
                // }
                todo!();
            }
        }
        None
    }

    fn title_view(&self) -> String {
        if !self.show_title {
            return String::new();
        }

        let mut view = String::new();

        // Show spinner if enabled
        if self.show_spinner {
            view.push_str(&self.spinner.view().to_string());
            view.push(' ');
        }

        let mut title_style = self.title_style.clone();
        title_style.content = self.title.clone();
        view.push_str(&title_style.content);

        // Add status message if there is one
        if !self.status_message.is_empty() {
            view.push_str("  ");
            view.push_str(&self.status_message);
        }

        view
    }

    fn status_view(&self) -> String {
        if !self.show_status_bar {
            return String::new();
        }

        let total_items = self.items.len();
        let visible_items = self.visible_items().len();

        let mut status = String::new();

        // Determine item name
        let item_name = if visible_items == 1 {
            &self.item_name_singular
        } else {
            &self.item_name_plural
        };

        // Show count
        if total_items == 0 {
            status.push_str(&format!("No {}", self.item_name_plural));
        } else {
            status.push_str(&format!("{} {}", visible_items, item_name));
        }

        let mut status_style = self.status_bar_style.clone();
        status_style.content = status;
        status_style.content
    }

    fn pagination_view(&self) -> String {
        if !self.show_pagination || self.total_pages <= 1 {
            return String::new();
        }

        let current_page = self.page + 1;
        format!("{}/{}", current_page, self.total_pages)
    }

    fn help_view(&self) -> String {
        if !self.show_help {
            return String::new();
        }

        "↑/↓:Navigate • q:Quit".to_string()
    }

    fn items_view<W: Write>(&self, w: &mut W) -> std::fmt::Result {
        let items = self.visible_items();

        if items.is_empty() {
            return write!(w, "No {}.", self.item_name_plural);
        }

        let start = self.page * self.per_page;
        let end = std::cmp::min(start + self.per_page, items.len());

        if start >= items.len() {
            return Ok(());
        }

        for (i, item) in items.iter().enumerate().take(end).skip(start) {
            if i > start {
                // Add spacing
                for _ in 0..self.delegate.spacing() {
                    writeln!(w)?;
                }
                // Add newline for the next item
                writeln!(w)?;
            }

            self.delegate.render(w, self, i, &**item);
        }

        Ok(())
    }

    pub fn with_infinite_scrolling(mut self, enabled: bool) -> Self {
        self.infinite_scrolling = enabled;
        self
    }

    pub fn update(&mut self, _event: Event) -> Option<Cmd> {
        None
    }
}

impl Model {
    pub fn render<W: Write>(&self, w: &mut W) -> Result<(), std::fmt::Error> {
        // Render title
        if self.show_title {
            writeln!(w, "{}", self.title_view())?;
        }

        // Render status
        if self.show_status_bar {
            writeln!(w, "{}", self.status_view())?;
        }

        // Render items
        self.items_view(w)?;

        // Render pagination
        if self.show_pagination && self.total_pages > 1 {
            writeln!(w, "\n{}", self.pagination_view())?;
        }

        // Render help
        if self.show_help {
            writeln!(w, "\n{}", self.help_view())?;
        }

        Ok(())
    }
}

impl MModel for Model {
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        (
            Self {
                width: input.size.0 as usize,
                height: input.size.1 as usize,
                ..self
            },
            None,
        )
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(key_event) = msg.downcast_ref::<KeyEvent>() {
            let mut new_self = self;
            let cmd = new_self.handle_key_event(key_event);
            return (new_self, cmd);
        }

        // Handle spinner tick messages
        if msg.downcast_ref::<TickMsg>().is_some() {
            let (new_spinner, cmd) = self.spinner.update(msg);
            return (
                Self {
                    spinner: new_spinner,
                    ..self
                },
                cmd,
            );
        }

        (self, None)
    }

    fn view(&self) -> impl Display {
        let mut output = String::new();
        let _ = self.render(&mut output);
        output
    }
}
