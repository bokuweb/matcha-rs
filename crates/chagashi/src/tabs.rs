use std::fmt::Display;

use unicode_width::UnicodeWidthStr;

use matcha::{
    batch, clamp_by, fill_by_space, remove_escape_sequences, style, Cmd, Color, InitInput, KeyCode,
    KeyEvent, Model, Msg, Stylize,
};

use matcha::DynModel;

/// A single tab: a title plus a child model that renders the content.
pub struct Tab {
    pub title: String,
    pub child: Box<dyn DynModel>,
}

impl Tab {
    pub fn new(title: impl Into<String>, child: Box<dyn DynModel>) -> Self {
        Self {
            title: title.into(),
            child,
        }
    }
}

/// A Bubble Tea "tabs" port: renders a tab strip and a window-like container below it.
///
/// - Keybinds: Left/Right, h/l, p/n, Tab/Shift+Tab
/// - Visual: 3-line rounded tabs with bottom join tweaks + top-less bordered window
pub struct Tabs {
    width: u16,
    tabs: Vec<Tab>,
    active: usize,
    highlight: Color,
    content_padding_y: u16,
}

impl Tabs {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            width: 0,
            tabs,
            active: 0,
            // bubbletea example uses AdaptiveColor; we pick the dark variant here.
            highlight: Color::Rgb {
                r: 0x7D,
                g: 0x56,
                b: 0xF4,
            },
            content_padding_y: 2,
        }
    }

    pub fn active(self, active: usize) -> Self {
        Self { active, ..self }
    }

    pub fn highlight(self, color: Color) -> Self {
        Self {
            highlight: color,
            ..self
        }
    }

    pub fn content_padding_y(self, padding: u16) -> Self {
        Self {
            content_padding_y: padding,
            ..self
        }
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    fn clamp_active(&mut self) {
        if self.tabs.is_empty() {
            self.active = 0;
            return;
        }
        self.active = std::cmp::min(self.active, self.tabs.len() - 1);
    }

    fn handle_key(&mut self, key: &KeyEvent) {
        if self.tabs.is_empty() {
            return;
        }
        match key.code {
            KeyCode::Right | KeyCode::Tab => {
                self.active = std::cmp::min(self.active + 1, self.tabs.len() - 1);
            }
            KeyCode::Left | KeyCode::BackTab => {
                self.active = self.active.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Char('n') => {
                self.active = std::cmp::min(self.active + 1, self.tabs.len() - 1);
            }
            KeyCode::Char('h') | KeyCode::Char('p') => {
                self.active = self.active.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn paint(&self, s: impl Into<String>) -> String {
        style(s.into()).with(self.highlight).to_string()
    }

    fn visible_width(s: &str) -> u16 {
        remove_escape_sequences(s).width() as u16
    }

    fn center_line(&self, line: String, width: u16) -> String {
        let line = clamp_by(&line, width);
        let w = Self::visible_width(&line);
        if w >= width {
            return line;
        }
        let left = (width - w) / 2;
        let right = width - w - left;
        format!(
            "{}{}{}",
            " ".repeat(left as usize),
            line,
            " ".repeat(right as usize)
        )
    }

    fn tab_block(
        &self,
        title: &str,
        is_active: bool,
        is_first: bool,
        is_last: bool,
    ) -> [String; 3] {
        // Rounded tab with 1-cell horizontal padding.
        let inner = format!(" {} ", title);
        let inner_w = Self::visible_width(&inner);

        let top = format!(
            "{}{}{}",
            self.paint("╭"),
            self.paint("─".repeat(inner_w as usize)),
            self.paint("╮")
        );
        let mid = format!("{}{}{}", self.paint("│"), inner, self.paint("│"));

        let (bl0, bm, br0) = if is_active {
            ("┘", " ", "└")
        } else {
            ("┴", "─", "┴")
        };

        // Bubble Tea example tweaks first/last joiners so the window below looks continuous.
        let bl = if is_first {
            if is_active {
                "│"
            } else {
                "├"
            }
        } else {
            bl0
        };
        let br = if is_last {
            if is_active {
                "│"
            } else {
                "┤"
            }
        } else {
            br0
        };

        let bottom = format!(
            "{}{}{}",
            self.paint(bl),
            self.paint(bm.repeat(inner_w as usize)),
            self.paint(br)
        );

        [top, mid, bottom]
    }

    fn tabs_row(&self) -> Vec<String> {
        if self.tabs.is_empty() {
            return vec![];
        }
        let mut blocks: Vec<[String; 3]> = Vec::with_capacity(self.tabs.len());
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == self.tabs.len() - 1;
            let is_active = i == self.active;
            blocks.push(self.tab_block(&tab.title, is_active, is_first, is_last));
        }

        let mut out = vec![String::new(), String::new(), String::new()];
        for b in blocks {
            out[0].push_str(&b[0]);
            out[1].push_str(&b[1]);
            out[2].push_str(&b[2]);
        }
        out
    }

    fn window_view(&self, content: &str, width: u16) -> Vec<String> {
        // width is total window width including borders. We remove top border like lipgloss.UnsetBorderTop().
        let total_w = width.max(2);
        let inner_w = total_w.saturating_sub(2);
        let side = self.paint("│");

        let mut lines: Vec<String> = Vec::new();
        for _ in 0..self.content_padding_y {
            lines.push(format!("{}{}{}", side, " ".repeat(inner_w as usize), side));
        }

        for raw in content.split('\n') {
            let centered = self.center_line(raw.to_string(), inner_w);
            let padded = fill_by_space(centered, inner_w);
            lines.push(format!("{}{}{}", side, padded, side));
        }

        for _ in 0..self.content_padding_y {
            lines.push(format!("{}{}{}", side, " ".repeat(inner_w as usize), side));
        }

        lines.push(format!(
            "{}{}{}",
            self.paint("└"),
            self.paint("─".repeat(inner_w as usize)),
            self.paint("┘")
        ));

        lines
    }
}

impl Model for Tabs {
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        let mut cmds = vec![];
        let mut tabs: Vec<Tab> = Vec::with_capacity(self.tabs.len());
        for tab in self.tabs.into_iter() {
            let (child, cmd) = tab.child.init_box(input);
            if let Some(cmd) = cmd {
                cmds.push(cmd);
            }
            tabs.push(Tab {
                title: tab.title,
                child,
            });
        }

        let cmd = if cmds.is_empty() {
            None
        } else {
            Some(batch(cmds))
        };

        let mut next = Self {
            width: input.size.0,
            tabs,
            ..self
        };
        next.clamp_active();
        (next, cmd)
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        let mut width = self.width;
        if let Some(r) = msg.downcast_ref::<matcha::ResizeEvent>() {
            width = r.0;
        }

        let mut next = Self { width, ..self };
        if let Some(key) = msg.downcast_ref::<KeyEvent>() {
            next.handle_key(key);
            next.clamp_active();
        }

        let mut cmds = vec![];
        let mut tabs: Vec<Tab> = Vec::with_capacity(next.tabs.len());
        for tab in next.tabs.into_iter() {
            let (child, cmd) = tab.child.update_box(msg);
            if let Some(cmd) = cmd {
                cmds.push(cmd);
            }
            tabs.push(Tab {
                title: tab.title,
                child,
            });
        }
        next.tabs = tabs;

        let cmd = if cmds.is_empty() {
            None
        } else {
            Some(batch(cmds))
        };
        (next, cmd)
    }

    fn view(&self) -> impl Display {
        if self.tabs.is_empty() {
            return String::new();
        }

        let mut rows = self.tabs_row();
        let row_width = rows
            .iter()
            .map(|l| Self::visible_width(l))
            .max()
            .unwrap_or(0);

        // Render active tab content inside a top-less bordered window.
        let active = std::cmp::min(self.active, self.tabs.len() - 1);
        let content = self.tabs[active].child.view_string();
        let window = self.window_view(&content, row_width);

        rows.extend(window);
        rows.join("\n")
    }
}
