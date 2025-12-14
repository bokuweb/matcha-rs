use std::fmt::Display;

use unicode_segmentation::UnicodeSegmentation;

use matcha::{batch, Cmd, Color, KeyCode, KeyEvent, KeyModifiers, Model, Msg, Stylize};

use crate::cursor;
use crate::utils::*;

/// A single-line text input component.
///
/// This widget tracks a cursor position and handles basic editing keys.
pub struct TextInput {
    prompt: String,
    placeholder: String,
    cursor: cursor::Cursor,
    value: String,
    focus: bool,
    pos: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            prompt: "> ".to_string(),
            placeholder: String::default(),
            cursor: cursor::Cursor::new(),
            value: String::default(),
            focus: false,
            pos: 0,
        }
    }
}

impl TextInput {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Create a new text input with default settings.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the placeholder text shown when the value is empty.
    pub fn set_placeholder(self, placeholder: impl Into<String>) -> Self {
        let placeholder = placeholder.into();
        let cursor = if !placeholder.is_empty() && self.value.is_empty() {
            let c: String = placeholder
                .graphemes(true)
                .next()
                .expect("placeholder should not be empty")
                .into();

            self.cursor
                .set_char(c)
                .set_text_color(Color::AnsiValue(240))
        } else {
            self.cursor
        };
        Self {
            cursor,
            placeholder,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Focus the input (enables editing) and start cursor blinking.
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
    /// Replace the internal cursor model.
    pub fn set_cursor(self, cursor: cursor::Cursor) -> Self {
        Self { cursor, ..self }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the cursor position (grapheme index) within the value.
    pub fn set_pos(self, pos: usize) -> Self {
        Self { pos, ..self }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Set the input value.
    pub fn set_value(self, value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            ..self
        }
    }

    /// cursor_start moves the cursor to the start of the input field.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn cursor_start(self) -> Self {
        if self.value.is_empty() {
            return self;
        }
        Self { pos: 0, ..self }
    }

    /// cursor_end moves the cursor to the end of the input field.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn cursor_end(self) -> Self {
        if self.value.is_empty() {
            return self;
        }
        let cur = self.cursor.set_char(" ");
        Self {
            cursor: cur,
            pos: self.value.len(),
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete the character under the cursor.
    pub fn delete_forward_char(self) -> Self {
        if self.pos >= self.value.len() || !self.focus {
            return self;
        }
        let value = self.value;
        let value = remove_char(value, self.pos);
        Self { value, ..self }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete the character before the cursor.
    pub fn delete_back_char(self) -> Self {
        if self.pos == 0 || !self.focus {
            return self;
        }
        let value = self.value;
        let pos = self.pos.saturating_sub(1);
        let value = remove_char(value, pos);
        Self { value, pos, ..self }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Move the cursor one grapheme to the left.
    pub fn move_left(self) -> Self {
        if self.focus {
            return self;
        }
        let pos = self.pos.saturating_sub(1);
        let cursor = self.cursor.set_char(
            self.value
                .graphemes(true)
                .nth(pos)
                .unwrap_or(" ")
                .to_string(),
        );
        Self { cursor, ..self }.set_pos(pos)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Move the cursor one grapheme to the right.
    pub fn move_right(self) -> Self {
        if self.focus {
            return self;
        }
        let pos = std::cmp::min(
            self.pos.saturating_add(1),
            self.value.graphemes(true).count(),
        );
        let cursor = self.cursor.set_char(
            self.value
                .graphemes(true)
                .nth(pos)
                .unwrap_or(" ")
                .to_string(),
        );
        Self { cursor, ..self }.set_pos(pos)
    }

    /// placeholderView returns the prompt and placeholder view, if any.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn placeholder_view(&self) -> String {
        let placeholder = &self.placeholder[1..];
        let placeholder = placeholder.with(Color::AnsiValue(240)).to_string();
        self.prompt.clone() + &format!("{}", self.cursor.view()) + &placeholder
    }
}

impl Model for TextInput {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if !self.focus {
            return (self, None);
        }

        let old_pos = self.pos;
        let mut cmds: matcha::BatchMsg = vec![];

        let (new_self, cmd) = if let Some(msg) = msg.downcast_ref::<KeyEvent>() {
            let (new_self, cmd) = if let KeyModifiers::CONTROL = msg.modifiers {
                match msg.code {
                    KeyCode::Char('a') => (self.cursor_start(), None),
                    KeyCode::Char('b') => (self.move_left(), None),
                    KeyCode::Char('d') => (self.delete_forward_char(), None),
                    KeyCode::Char('e') => (self.cursor_end(), None),
                    KeyCode::Char('h') => (self.delete_back_char(), None),
                    KeyCode::Char('f') => (self.move_right(), None),
                    _ => (self, None),
                }
            } else {
                match msg.code {
                    KeyCode::Backspace => (self.delete_back_char(), None),
                    KeyCode::Delete => (self.delete_forward_char(), None),
                    KeyCode::Left => (self.move_left(), None),
                    KeyCode::Right => (self.move_right(), None),
                    KeyCode::Char(char) => {
                        let value = self.value;
                        let value = insert_char(value, self.pos, char);

                        let c = value
                            .graphemes(true)
                            .nth(self.pos + 1)
                            .unwrap_or(" ")
                            .to_string();
                        let cursor = self.cursor.set_char(c).reset_text_color();
                        let pos = std::cmp::min(value.graphemes(true).count(), self.pos + 1);
                        (
                            Self {
                                value,
                                cursor,
                                pos,
                                ..self
                            },
                            None,
                        )
                    }
                    _ => (self, None),
                }
            };
            (new_self, cmd)
        } else {
            (self, None)
        };

        if let Some(cmd) = cmd {
            cmds.push(cmd);
        }

        let cur = if new_self.value.is_empty() && !new_self.placeholder.is_empty() {
            let c: String = new_self.placeholder.graphemes(true).next().unwrap().into();
            new_self
                .cursor
                .set_char(c)
                .set_text_color(Color::AnsiValue(240))
        } else {
            new_self.cursor
        };

        let (cur, cmd) = cur.update(msg);
        if let Some(cmd) = cmd {
            cmds.push(cmd);
        }

        let cursor = if old_pos != new_self.pos {
            let cur = cur.set_blink(false);
            let (cur, cmd) = cur.blink_cmd();
            if let Some(cmd) = cmd {
                cmds.push(cmd);
            };
            cur
        } else {
            cur
        };
        (Self { cursor, ..new_self }, Some(batch(cmds)))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn view(&self) -> impl Display {
        if self.value.is_empty() && !self.placeholder.is_empty() {
            return self.placeholder_view();
        }
        let value = self.value.clone();

        if self.pos == 0 {
            let (_, tail) = split_at(value, 1);
            return self.prompt.clone() + &format!("{}", self.cursor.view()) + &tail;
        }
        if self.pos < self.value.len() {
            let (head, tail) = split_at(value, self.pos);
            let tail = if tail.is_empty() {
                tail
            } else {
                let (_, tail) = split_at(tail, 1);
                tail
            };

            return self.prompt.clone() + &head + &format!("{}", self.cursor.view()) + &tail;
        }

        if self.focus {
            self.prompt.clone() + &self.value + &format!("{}", self.cursor.view())
        } else {
            self.prompt.clone() + &self.value
        }
    }
}
