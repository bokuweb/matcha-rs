use super::Position;
use super::Row;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn with_rows(rows: impl Into<Vec<Row>>) -> Self {
        Self { rows: rows.into() }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    // pub fn is_empty(&self) -> bool {
    //     self.rows.is_empty()
    // }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
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
