use std::{fmt::Display, rc::Rc};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Wrap a string into lines with a maximum display width.
///
/// This function is *ANSI-aware*: it tries to preserve ANSI escape sequences without
/// counting them towards the displayed width.
///
/// The return value is a vector of lines (without trailing `\n`).
pub fn wrap(s: &str, max_width: u16) -> Vec<String> {
    let mut width: u16 = 0;
    let mut index = 0;
    let mut result = vec![String::new()];
    let mut graphemes = s.graphemes(true);

    while let Some(grapheme) = graphemes.next() {
        if grapheme == "\x1b" {
            result[index].push_str(grapheme);
            // `[`
            if let Some(grapheme) = graphemes.next() {
                result[index].push_str(grapheme);
            }
            #[allow(clippy::while_let_on_iterator)]
            while let Some(grapheme) = graphemes.next() {
                result[index].push_str(grapheme);
                if matches!(
                    grapheme.as_bytes().first(),
                    Some(0x40..=0x5c) | Some(0x61..=0x7a)
                ) {
                    break;
                }
            }
        } else {
            let grapheme_width = grapheme.width() as u16;
            if width + grapheme_width > max_width {
                index += 1;
                result.push(String::new());
                width = 0;
            }
            result[index].push_str(grapheme);
            width += grapheme_width;
        }
    }
    result
}

/// Clamp a string to a maximum display width.
///
/// This function is *ANSI-aware*: it preserves escape sequences while ensuring the
/// visible grapheme width does not exceed `max_width`.
pub fn clamp_by(s: &str, max_width: u16) -> String {
    let mut width: u16 = 0;
    let mut result = String::new();

    let mut graphemes = s.graphemes(true);
    let mut clamped = false;

    while let Some(grapheme) = graphemes.next() {
        if grapheme == "\x1b" {
            result.push_str(grapheme);
            // `[`
            if let Some(grapheme) = graphemes.next() {
                result.push_str(grapheme);
            }
            #[allow(clippy::while_let_on_iterator)]
            while let Some(grapheme) = graphemes.next() {
                result.push_str(grapheme);
                if matches!(
                    grapheme.as_bytes().first(),
                    Some(0x40..=0x5c) | Some(0x61..=0x7a)
                ) {
                    break;
                }
            }
        } else {
            if clamped {
                continue;
            }
            let grapheme_width = grapheme.width() as u16;
            if width + grapheme_width > max_width {
                clamped = true;
            } else {
                result.push_str(grapheme);
                width += grapheme_width;
            }
        }
    }
    result
}

/// Pad `target` with spaces so its visible width becomes `max_width`.
///
/// The width calculation ignores ANSI escape sequences.
pub fn fill_by_space(target: String, max_width: u16) -> String {
    let d = max_width.saturating_sub(remove_escape_sequences(&target).width() as u16);
    if d != 0 {
        format!("{}{}", target, " ".repeat(d as usize))
    } else {
        target
    }
}

/// Remove ANSI escape sequences from `text`.
///
/// This is useful when you need to measure the "visible" width of styled strings.
pub fn remove_escape_sequences(text: &str) -> String {
    let mut result = String::new();
    let mut graphemes = text.graphemes(true);

    while let Some(g) = graphemes.next() {
        if g == "\x1b" {
            if let Some(grapheme) = graphemes.next() {
                if grapheme != "[" {
                    break;
                }
            }
            #[allow(clippy::while_let_on_iterator)]
            while let Some(grapheme) = graphemes.next() {
                if matches!(
                    grapheme.as_bytes().first(),
                    Some(0x40..=0x5c) | Some(0x61..=0x7a)
                ) {
                    break;
                }
            }
        } else {
            result += g;
        }
    }
    result
}

/// Format a view for the given terminal size.
///
/// - Truncates to the last `height` lines
/// - Clamps each line to `width` and right-pads with spaces
/// - Joins lines using `\r\n` for terminal-friendly rendering
pub fn format(view: impl Display, size: (u16, u16)) -> String {
    let view = view.to_string();
    let splitted: Rc<[&str]> = view.split('\n').rev().collect();
    splitted
        .iter()
        .take(size.1 as usize)
        .map(|l| fill_by_space(clamp_by(l, size.0), size.0))
        .rev()
        .collect::<Vec<String>>()
        .join("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_hello_with_escape_sequences() {
        let input = "\x1b[31mHello, World!\x1b[0m"; // Example with escape sequences
        let max_length = 10;
        let clamped = clamp_by(input, max_length);
        assert_eq!(clamped, "\x1b[31mHello, Wor\x1b[0m");
    }

    #[test]
    fn test_clamp_kana_with_escape_sequences() {
        let input = "\x1b[31mこんにちは!いい天気ですね\x1b[0m"; // Example with escape sequences
        let max_length = 10;
        let clamped = clamp_by(input, max_length);
        assert_eq!(clamped, "\x1b[31mこんにちは\x1b[0m");
    }

    #[test]
    fn test_clamp_kana_with_nested_escape_sequences() {
        let input = "\x1b[31mこんに\x1b[31mち\x1b[0mは!いい天気ですね\x1b[0m"; // Example with escape sequences
        let max_length = 10;
        let clamped = clamp_by(input, max_length);
        assert_eq!(clamped, "\x1b[31mこんに\x1b[31mち\x1b[0mは\x1b[0m");
    }

    #[test]
    fn test_remove_escape_sequences() {
        let input = "\x1b[31mこんに\x1b[31mち\x1b[0mは!いい天気ですね\x1b[0m"; // Example with escape sequences
        let removed = remove_escape_sequences(input);
        assert_eq!(removed, "こんにちは!いい天気ですね");
    }
}
