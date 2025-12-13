use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug)]
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
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
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
    pub fn len(&self) -> usize {
        self.len
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
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
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
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
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}
