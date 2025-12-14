//! A multi-line text editor (textarea) component.
//!
//! This module provides [`Textarea`], a basic editable buffer with cursor movement,
//! insertion/deletion and optional borders.

mod document;
mod position;
mod row;

use std::fmt::Display;

use unicode_segmentation::UnicodeSegmentation;

use document::Document;
use matcha::{key, Cmd, InitInput, KeyCode, KeyEvent, Model};
use position::Position;
use row::Row;

use crate::{
    borderize::{BorderOption, Borderize},
    cursor::{self, Cursor},
    utils::split_at,
};

/// KeyMap defines the keybindings for the viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Key actions recognized by [`Textarea`].
pub enum TextareaKeys {
    /// Move cursor left.
    MoveLeft,
    /// Move cursor right.
    MoveRight,
    /// Move cursor up.
    MoveUp,
    /// Move cursor down.
    MoveDown,
    /// Insert a newline.
    InsertNewline,
    /// Delete the character before the cursor.
    DeleteBack,
    /// Delete the character under the cursor.
    DeleteForward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Default keybindings for [`Textarea`].
pub struct Keybindings(matcha::KeyBindings<TextareaKeys>);

impl Default for Keybindings {
    fn default() -> Self {
        let bindings = [
            (key!(ctrl - b), TextareaKeys::MoveLeft),
            (key!(left), TextareaKeys::MoveLeft),
            (key!(ctrl - f), TextareaKeys::MoveRight),
            (key!(right), TextareaKeys::MoveRight),
            (key!(ctrl - p), TextareaKeys::MoveUp),
            (key!(up), TextareaKeys::MoveUp),
            (key!(ctrl - n), TextareaKeys::MoveDown),
            (key!(down), TextareaKeys::MoveDown),
            (key!(enter), TextareaKeys::InsertNewline),
            (key!(ctrl - m), TextareaKeys::InsertNewline),
            (key!(backspace), TextareaKeys::DeleteBack),
            (key!(ctrl - h), TextareaKeys::DeleteBack),
            (key!(delete), TextareaKeys::DeleteForward),
            (key!(ctrl - d), TextareaKeys::DeleteForward),
        ]
        .into_iter()
        .collect();
        Keybindings(matcha::KeyBindings::new(bindings))
    }
}

/// A multi-line text editor component.
///
/// `Textarea` is a thin wrapper around an internal model and optional borders.
pub struct Textarea(Borderize<Inner>);

impl Default for Textarea {
    fn default() -> Self {
        Self(Borderize::new(Inner::new()))
    }
}

impl Model for Textarea {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn init(self, input: &matcha::InitInput) -> (Self, Option<matcha::Cmd>) {
        let (child, cmd) = self.0.child.init(input);
        (Self(Borderize { child, ..self.0 }), cmd)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn update(self, msg: &matcha::Msg) -> (Self, Option<matcha::Cmd>) {
        let (child, cmd) = self.0.child.update(msg);
        (Self(Borderize { child, ..self.0 }), cmd)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn view(&self) -> impl Display {
        self.0.view()
    }
}

impl Textarea {
    /// Create a new empty textarea.
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the textarea size.
    pub fn size(self, width: u16, height: u16) -> Self {
        let child = self.0.child.size(width, height);
        Self(Borderize { child, ..self.0 })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the textarea width.
    pub fn width(self, width: u16) -> Self {
        let child = self.0.child.width(width);
        Self(Borderize { child, ..self.0 })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the textarea height.
    pub fn height(self, height: u16) -> Self {
        let child = self.0.child.height(height);
        Self(Borderize { child, ..self.0 })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Focus the textarea (enables editing) and starts cursor blinking.
    pub fn focus(self) -> (Self, Option<Cmd>) {
        let (child, cmd) = self.0.child.focus();
        (Self(Borderize { child, ..self.0 }), cmd)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Create a textarea initialized with the given content.
    pub fn with_content(content: impl Into<String>) -> Self {
        let child = Inner::with_content(content);
        Self(Borderize::new(child))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Enable a simple left border.
    pub fn border(self) -> Self {
        Self(self.0.left(BorderOption {
            show: true,
            ..BorderOption::default()
        }))
    }
}

/// Internal textarea implementation.
///
/// This type handles editing behavior and rendering; it is wrapped by [`Textarea`].
pub struct Inner {
    // placeholder: String,
    width: u16,
    height: u16,
    document: Document,
    cursor: cursor::Cursor,
    focus: bool,
    offset: Position,
    cursor_position: Position,
    key_bindings: Keybindings,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            // placeholder: String::default(),
            width: 0,
            height: 0,
            document: Document::default(),
            cursor: cursor::Cursor::new(),
            focus: false,
            offset: Position::new(0, 0),
            cursor_position: Position::new(0, 0),
            key_bindings: Keybindings::default(),
        }
    }
}

impl Inner {
    /// Create a new empty inner textarea model.
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the inner textarea size.
    pub fn size(self, width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the inner textarea width.
    pub fn width(self, width: u16) -> Self {
        Self { width, ..self }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the inner textarea height.
    pub fn height(self, height: u16) -> Self {
        Self { height, ..self }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Focus the textarea for editing.
    pub fn focus(self) -> (Self, Option<Cmd>) {
        let cursor = self.cursor.focus();
        (
            Self {
                cursor: cursor.0,
                focus: true,
                ..self
            },
            cursor.1,
        )
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Create an inner textarea initialized with the given content.
    pub fn with_content(content: impl Into<String>) -> Self {
        let mut rows = Vec::new();
        for value in content.into().lines() {
            rows.push(Row::from(value));
        }

        let cursor = Self::set_cursor_char(Position::new(0, 0), cursor::Cursor::new(), &rows);

        Self {
            document: Document::with_rows(rows),
            cursor,
            ..Default::default()
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn set_cursor_char(cursor_position: Position, cursor: Cursor, rows: &[Row]) -> Cursor {
        let Position { x, y } = cursor_position;
        let c: String = rows
            .get(y)
            .expect("rows should not be empty")
            .as_str()
            .graphemes(true)
            .nth(x)
            .unwrap_or(" ")
            .into();
        cursor.set_char(c)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn render_row(&self, row: &Row, index: usize) -> String {
        let start = self.offset.x;
        // sub numbering
        let end = self
            .offset
            .x
            .saturating_add(self.width as usize)
            // line number
            .saturating_sub(4);

        let s = row.render(start, end);
        if self.cursor_position.y != index {
            return s;
        }

        let cursor_x = self.cursor_position.x.saturating_sub(start);

        if cursor_x == 0 {
            let (_, tail) = split_at(s, 1);
            return format!("{}", self.cursor.view()) + &tail;
        }

        if cursor_x < s.len() {
            let (head, tail) = split_at(s, cursor_x);
            let tail = if tail.is_empty() {
                tail
            } else {
                let (_, tail) = split_at(tail, 1);
                tail
            };
            return head + &format!("{}", self.cursor.view()) + &tail;
        }

        if self.focus {
            s + &format!("{}", self.cursor.view())
        } else {
            s
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn render_rows(&self) -> String {
        let height = self.height;
        let mut rows = vec![];
        for row in 0..height {
            let mut s = String::default();
            let n = self.offset.y.saturating_add(row as usize);
            if let Some(row) = self.document.row(n) {
                s += &format!("{:>3} ", n.saturating_add(1));
                s += &self.render_row(row, n);
            } else {
                s += &format!("{:>1} ~", " ");
            }
            rows.push(s);
        }
        rows.join("\n")
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn move_right(self) -> Self {
        let Position { y, x } = self.cursor_position;
        let height = self.document.len();
        let width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        let (x, y) = if x < width {
            (x + 1, y)
        } else if y < height && self.document.row(y + 1).is_some() {
            (0, y + 1)
        } else {
            (x, y)
        };

        let cursor_position = Position::new(x, y);
        let cursor = Self::set_cursor_char(cursor_position, self.cursor, self.document.rows());

        Self {
            cursor_position,
            cursor,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn move_left(self) -> Self {
        let Position { y, x } = self.cursor_position;
        let (x, y) = if x > 0 {
            (x.saturating_sub(1), y)
        } else if y > 0 {
            if let Some(row) = self.document.row(y.saturating_sub(1)) {
                (row.len(), y.saturating_sub(1))
            } else {
                (0, y.saturating_sub(1))
            }
        } else {
            (x, y)
        };

        let cursor_position = Position::new(x, y);
        let cursor = Self::set_cursor_char(cursor_position, self.cursor, self.document.rows());

        Self {
            cursor_position,
            cursor,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn move_up(self) -> Self {
        let Position { mut y, mut x } = self.cursor_position;
        y = y.saturating_sub(1);
        if let Some(row) = self.document.row(y) {
            let n = row.as_str().graphemes(true).count();
            if x >= n {
                x = n;
            }
        }
        let cursor_position = Position::new(x, y);
        let cursor = Self::set_cursor_char(cursor_position, self.cursor, self.document.rows());

        Self {
            cursor_position,
            cursor,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn move_down(self) -> Self {
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        if let Some(next) = self.document.row(y + 1) {
            if y < height {
                y = y.saturating_add(1);
                let n = next.as_str().graphemes(true).count();
                if x >= n {
                    x = n;
                }
            }
        }
        let cursor_position = Position::new(x, y);
        let cursor = Self::set_cursor_char(cursor_position, self.cursor, self.document.rows());

        Self {
            cursor_position,
            cursor,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn insert(self, c: char) -> Self {
        let document = self.document.insert(&self.cursor_position, c);
        let cursor = Self::set_cursor_char(self.cursor_position, self.cursor, document.rows());
        Self {
            document,
            cursor,
            cursor_position: Position::new(
                self.cursor_position.x.saturating_add(1),
                self.cursor_position.y,
            ),
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn insert_newline(self) -> Self {
        let document = self.document.insert_newline(&self.cursor_position);
        Self {
            document,
            cursor_position: Position::new(0, self.cursor_position.y.saturating_add(1)),
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete the character before the cursor.
    pub fn delete_back(self) -> Self {
        if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
            let new_self = self.move_left();
            let document = new_self.document.delete(&new_self.cursor_position);
            let cursor =
                Self::set_cursor_char(new_self.cursor_position, new_self.cursor, document.rows());
            Self {
                document,
                cursor,
                ..new_self
            }
        } else {
            self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete the character under the cursor.
    pub fn delete_forward(self) -> Self {
        let document = self.document.delete(&self.cursor_position);
        let cursor = Self::set_cursor_char(self.cursor_position, self.cursor, document.rows());
        Self {
            document,
            cursor,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn scroll(self) -> Self {
        let Position { x, y } = self.cursor_position;
        let width = self.width as usize - 4;
        let height = self.height as usize;
        let mut offset = self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
        Self { offset, ..self }
    }
}

impl Model for Inner {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        (
            Self {
                width: input.size.0,
                height: input.size.1,
                ..self
            },
            None,
        )
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn update(self, msg: &matcha::Msg) -> (Self, Option<matcha::Cmd>) {
        let mut cmds: matcha::BatchMsg = vec![];
        let old_cursor = self.cursor_position;
        let new_self = if let Some(event) = msg.downcast_ref::<KeyEvent>() {
            let key = self.key_bindings.0.get(matcha::Key::from(event));
            match key {
                Some(TextareaKeys::MoveLeft) => self.move_left(),
                Some(TextareaKeys::MoveRight) => self.move_right(),
                Some(TextareaKeys::MoveUp) => self.move_up(),
                Some(TextareaKeys::MoveDown) => self.move_down(),
                Some(TextareaKeys::InsertNewline) => self.insert_newline(),
                Some(TextareaKeys::DeleteBack) => self.delete_back(),
                Some(TextareaKeys::DeleteForward) => self.delete_forward(),
                _ => match event.code {
                    KeyCode::Char(char) => self.insert(char),
                    _ => self,
                },
            }
        } else {
            self
        };
        let new_self = new_self.scroll();
        let new_cursor = if new_self.cursor_position != old_cursor {
            let (new_cursor, cmd) = new_self.cursor.blink_cmd();
            let new_cursor = new_cursor.set_blink(false);
            cmd.into_iter().for_each(|c| cmds.push(c));
            new_cursor
        } else {
            new_self.cursor
        };

        let (cursor, cmd) = new_cursor.update(msg);
        cmd.into_iter().for_each(|c| cmds.push(c));
        (Self { cursor, ..new_self }, Some(matcha::batch(cmds)))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn view(&self) -> impl Display {
        self.render_rows()
    }
}
