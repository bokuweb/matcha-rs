use super::Position;
use super::Row;

#[derive(Default)]
/// Text buffer used by [`super::Textarea`].
///
/// The document is represented as a vector of rows.
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Create a document from a list of rows.
    pub fn with_rows(rows: impl Into<Vec<Row>>) -> Self {
        Self { rows: rows.into() }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Get a row by index.
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Borrow all rows.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    // pub fn is_empty(&self) -> bool {
    //     self.rows.is_empty()
    // }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Number of rows in the document.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Insert a character at a cursor position.
    pub fn insert(self, at: &Position, c: char) -> Self {
        if at.y > self.rows.len() {
            return self;
        }
        if at.y == self.rows.len() {
            // let mut row = Row::default();
            // row.insert(0, c);
            // self.rows.push(row);
            self
        } else {
            let mut rows = self.rows;
            if let Some(row) = rows.get_mut(at.y) {
                row.insert(at.x, c);
            }
            Self { rows }
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Insert a newline at a cursor position.
    pub fn insert_newline(self, at: &Position) -> Self {
        if at.y > self.rows.len() {
            return self;
        }
        let len = self.rows.len();
        let mut rows = self.rows;
        if at.y == len {
            rows.push(Row::default());
            return Self { rows };
        }
        let mut current_row = std::mem::take(&mut rows[at.y]);
        let new_row = current_row.split(at.x);
        rows.insert(at.y + 1, new_row);
        rows[at.y] = current_row;
        Self { rows }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Delete a character at a cursor position.
    ///
    /// If the cursor is at end-of-line and there is a next row, the next row is appended.
    pub fn delete(self, at: &Position) -> Self {
        let len = self.rows.len();
        if at.y >= len {
            return self;
        }
        let mut rows = self.rows;
        if at.x == rows[at.y].len() && at.y + 1 < len {
            let next_row = rows.remove(at.y + 1);
            let row = &mut rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut rows[at.y];
            row.delete(at.x);
        }
        Self { rows }
    }
}

#[cfg(test)]
mod tests {
    use super::{Document, Position, Row};
    use proptest::prelude::*;
    use unicode_segmentation::UnicodeSegmentation;

    fn grapheme_len(value: &str) -> usize {
        value.graphemes(true).count()
    }

    fn doc_from_strings(rows: &[String]) -> Document {
        let rows = rows.iter().map(|s| Row::from(s.as_str())).collect::<Vec<_>>();
        Document::with_rows(rows)
    }

    fn doc_to_strings(doc: &Document) -> Vec<String> {
        doc.rows().iter().map(|r| r.as_str().to_string()).collect()
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

    proptest! {
        #[test]
        fn insert_into_existing_row_matches_reference(
            rows in proptest::collection::vec(any::<String>(), 1..6),
            y in any::<usize>(),
            x in any::<usize>(),
            c in any::<char>(),
        ) {
            let row_index = y % rows.len();
            let mut expected = rows.clone();
            let x = {
                let len = grapheme_len(&expected[row_index]);
                if len == 0 { 0 } else { x % (len + 1) }
            };
            expected[row_index] = reference_insert(&expected[row_index], x, c);

            let doc = doc_from_strings(&rows);
            let doc = doc.insert(&Position::new(x, row_index), c);

            prop_assert_eq!(doc_to_strings(&doc), expected);
        }

        #[test]
        fn insert_newline_keeps_or_increments_row_count(
            rows in proptest::collection::vec(any::<String>(), 0..6),
            y in any::<usize>(),
            x in any::<usize>(),
        ) {
            let len = rows.len();
            let doc = doc_from_strings(&rows);

            let at = if len == 0 {
                Position::new(0, y % 2)
            } else {
                Position::new(x, y % (len + 2))
            };

            let updated = doc.insert_newline(&at);
            let expected_len = if at.y <= len { len + 1 } else { len };
            prop_assert_eq!(updated.len(), expected_len);
        }

        #[test]
        fn delete_at_end_of_row_merges_next_row(
            left in any::<String>(),
            right in any::<String>(),
        ) {
            let left_len = grapheme_len(&left);
            let rows = vec![left.clone(), right.clone()];
            let doc = doc_from_strings(&rows);

            let updated = doc.delete(&Position::new(left_len, 0));
            let actual = doc_to_strings(&updated);

            prop_assert_eq!(actual.len(), 1);
            prop_assert_eq!(actual[0].as_str(), format!("{left}{right}"));
        }

        #[test]
        fn insert_newline_then_delete_restores_original_row(
            rows in proptest::collection::vec(any::<String>(), 1..6),
            y in any::<usize>(),
            x in any::<usize>(),
        ) {
            let row_index = y % rows.len();
            let split_at = {
                let len = grapheme_len(&rows[row_index]);
                if len == 0 { 0 } else { x % (len + 1) }
            };

            let doc = doc_from_strings(&rows);
            let doc = doc.insert_newline(&Position::new(split_at, row_index));
            let doc = doc.delete(&Position::new(split_at, row_index));

            prop_assert_eq!(doc_to_strings(&doc), rows);
        }
    }
}
