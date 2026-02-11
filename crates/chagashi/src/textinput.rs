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
        let max = self.value.graphemes(true).count();
        Self {
            pos: std::cmp::min(pos, max),
            ..self
        }
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
            pos: self.value.graphemes(true).count(),
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete the character under the cursor.
    pub fn delete_forward_char(self) -> Self {
        if self.pos >= self.value.graphemes(true).count() || !self.focus {
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
        if !self.focus {
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
        if !self.focus {
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
        let (_, placeholder) = split_at(self.placeholder.clone(), 1);
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
        if self.pos < self.value.graphemes(true).count() {
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

#[cfg(test)]
mod tests {
    use super::TextInput;
    use crate::cursor::{Cursor, CursorMode};
    use crate::utils::{insert_char, remove_char};
    use matcha::{KeyCode, KeyEvent, KeyModifiers, Model, Msg};
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;
    use unicode_segmentation::UnicodeSegmentation;

    fn grapheme_len(value: &str) -> usize {
        value.graphemes(true).count()
    }

    fn key_msg(code: KeyCode) -> Msg {
        Box::new(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn focused_input(value: String, pos: usize) -> TextInput {
        let (cursor, _) = Cursor::new().set_mode(CursorMode::Static);
        let (input, _) = TextInput::new()
            .set_cursor(cursor)
            .set_value(value)
            .set_pos(pos)
            .focus();
        input
    }

    #[derive(Clone, Debug)]
    enum Op {
        Left,
        Right,
        Backspace,
        Delete,
        Insert(char),
    }

    fn apply_ref(mut value: String, mut pos: usize, op: &Op) -> (String, usize) {
        match op {
            Op::Left => {
                pos = pos.saturating_sub(1);
            }
            Op::Right => {
                pos = std::cmp::min(pos.saturating_add(1), grapheme_len(&value));
            }
            Op::Backspace => {
                if pos > 0 {
                    let remove_at = pos.saturating_sub(1);
                    value = remove_char(value, remove_at);
                    pos = remove_at;
                }
            }
            Op::Delete => {
                if pos < grapheme_len(&value) {
                    value = remove_char(value, pos);
                }
            }
            Op::Insert(c) => {
                value = insert_char(value, pos, *c);
                pos = std::cmp::min(grapheme_len(&value), pos.saturating_add(1));
            }
        }
        (value, pos)
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            fork: false,
            .. ProptestConfig::default()
        })]

        #[test]
        fn insert_then_backspace_restores_original(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
            c in proptest::char::range(' ', '~'),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };
            let input = focused_input(value.clone(), index);

            let insert = key_msg(KeyCode::Char(c));
            let (input, _) = input.update(&insert);

            let backspace = key_msg(KeyCode::Backspace);
            let (input, _) = input.update(&backspace);

            prop_assert_eq!(input.value, value);
            prop_assert_eq!(input.pos, index);
        }

        #[test]
        fn char_input_on_focus_increases_grapheme_len(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
            c in proptest::char::range(' ', '~'),
        ) {
            let before_len = grapheme_len(&value);
            let index = if before_len == 0 { 0 } else { at % (before_len + 1) };
            let input = focused_input(value, index);

            let insert = key_msg(KeyCode::Char(c));
            let (input, _) = input.update(&insert);

            prop_assert_eq!(grapheme_len(&input.value), before_len + 1);
            prop_assert_eq!(input.pos, index + 1);
        }

        #[test]
        fn input_is_noop_when_not_focused(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
            c in proptest::char::range(' ', '~'),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };
            let input = TextInput::new().set_value(value.clone()).set_pos(index);

            let insert = key_msg(KeyCode::Char(c));
            let (updated, _) = input.update(&insert);

            prop_assert_eq!(updated.value, value);
            prop_assert_eq!(updated.pos, index);
        }

        #[test]
        fn operation_sequence_matches_reference_model(
            initial in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
            ops in proptest::collection::vec(
                prop_oneof![
                    Just(Op::Left),
                    Just(Op::Right),
                    Just(Op::Backspace),
                    Just(Op::Delete),
                    proptest::char::range(' ', '~').prop_map(Op::Insert),
                ],
                0..64
            ),
        ) {
            let len = grapheme_len(&initial);
            let index = if len == 0 { 0 } else { at % (len + 1) };

            let mut input = focused_input(initial.clone(), index);
            let mut expected_value = initial;
            let mut expected_pos = index;

            for op in &ops {
                let msg = match op {
                    Op::Left => key_msg(KeyCode::Left),
                    Op::Right => key_msg(KeyCode::Right),
                    Op::Backspace => key_msg(KeyCode::Backspace),
                    Op::Delete => key_msg(KeyCode::Delete),
                    Op::Insert(c) => key_msg(KeyCode::Char(*c)),
                };
                let (next, _) = input.update(&msg);
                input = next;

                (expected_value, expected_pos) = apply_ref(expected_value, expected_pos, op);
            }

            prop_assert_eq!(input.value.as_str(), expected_value.as_str());
            prop_assert_eq!(input.pos, expected_pos);
            prop_assert!(input.pos <= grapheme_len(&input.value));
        }
    }
}
