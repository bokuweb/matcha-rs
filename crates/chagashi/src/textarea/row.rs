use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug)]
/// A single line of text stored as graphemes.
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Render a slice of the row from grapheme index `start` to `end`.
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.len);
        let start = cmp::min(start, end);
        let mut result = String::new();
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                if c == '\t' {
                    result += " ";
                } else {
                    result.push(c);
                }
            }
        }
        result
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Return the grapheme length of the row.
    pub fn len(&self) -> usize {
        self.len
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Insert a character at grapheme index `at`.
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len = length;
        self.string = result;
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete the grapheme at index `at`.
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                #[cfg(feature = "tracing")]
                tracing::trace!("delete char at {}", at);
                length += 1;
                result.push_str(grapheme);
            } else {
                #[cfg(feature = "tracing")]
                tracing::trace!("{} hit {}", at, grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Append `new` to the end of this row.
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Split the row at grapheme index `at` and return the tail part.
    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut splitted_row: String = String::new();
        let mut splitted_length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            len: splitted_length,
        }
    }

    // pub fn as_bytes(&self) -> &[u8] {
    //     self.string.as_bytes()
    // }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::Row;
    use proptest::prelude::*;
    use unicode_segmentation::UnicodeSegmentation;

    fn grapheme_len(value: &str) -> usize {
        value.graphemes(true).count()
    }

    fn reference_insert(value: &str, at: usize, c: char) -> String {
        let mut result = String::new();
        let mut inserted = false;
        for (idx, g) in value.graphemes(true).enumerate() {
            if idx == at {
                result.push(c);
                inserted = true;
            }
            result.push_str(g);
        }
        if !inserted {
            result.push(c);
        }
        result
    }

    fn reference_delete(value: &str, at: usize) -> String {
        value
            .graphemes(true)
            .enumerate()
            .filter_map(|(idx, g)| if idx == at { None } else { Some(g) })
            .collect()
    }

    proptest! {
        #[test]
        fn insert_matches_reference(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
            c in proptest::char::range(' ', '~'),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };
            let expected = reference_insert(&value, index, c);

            let mut row = Row::from(value.as_str());
            row.insert(index, c);

            prop_assert_eq!(row.as_str(), expected);
            prop_assert_eq!(row.len(), grapheme_len(row.as_str()));
        }

        #[test]
        fn delete_matches_reference(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };
            let expected = reference_delete(&value, index);

            let mut row = Row::from(value.as_str());
            row.delete(index);

            prop_assert_eq!(row.as_str(), expected);
            prop_assert_eq!(row.len(), grapheme_len(row.as_str()));
        }

        #[test]
        fn split_then_append_restores_original(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };

            let mut head = Row::from(value.as_str());
            let tail = head.split(index);
            head.append(&tail);

            prop_assert_eq!(head.as_str(), value);
            prop_assert_eq!(head.len(), grapheme_len(head.as_str()));
        }
    }
}
