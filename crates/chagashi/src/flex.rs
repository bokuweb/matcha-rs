use std::fmt::Display;

use matcha::{batch, fill_by_space, Cmd, InitInput, Model, Msg, ResizeEvent};

use crate::dyn_model::DynModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone)]
pub struct FlexOption {
    /// Minimum width per item. If the available width cannot satisfy this, the number of columns
    /// is reduced and items will wrap to the next row.
    pub min_item_width: u16,
    /// Horizontal gap between columns (in cells).
    pub gap: u16,
    /// Enable wrapping when the available width is insufficient.
    pub wrap: bool,
    /// Maximum number of columns. If not set, `children.len()` is used as the upper bound.
    pub columns: Option<u16>,
    /// Layout direction.
    pub direction: FlexDirection,
}

impl Default for FlexOption {
    fn default() -> Self {
        Self {
            min_item_width: 12,
            gap: 1,
            wrap: true,
            columns: None,
            direction: FlexDirection::Row,
        }
    }
}

/// A Flexbox-inspired layout component for terminal UIs.
///
/// - Renders children horizontally (row) or vertically (column)
/// - When `wrap=true` in row mode, it reduces the number of columns based on terminal width
/// - Uses `matcha::formatter` utilities for width-aware clamp/padding
pub struct Flex {
    width: u16,
    opt: FlexOption,
    children: Vec<Box<dyn DynModel>>,
}

impl Flex {
    pub fn new(children: Vec<Box<dyn DynModel>>) -> Self {
        Self {
            width: 0,
            opt: FlexOption::default(),
            children,
        }
    }

    pub fn options(self, opt: FlexOption) -> Self {
        Self { opt, ..self }
    }

    pub fn gap(self, gap: u16) -> Self {
        Self {
            opt: FlexOption { gap, ..self.opt },
            ..self
        }
    }

    pub fn min_item_width(self, w: u16) -> Self {
        Self {
            opt: FlexOption {
                min_item_width: w,
                ..self.opt
            },
            ..self
        }
    }

    pub fn wrap(self, wrap: bool) -> Self {
        Self {
            opt: FlexOption { wrap, ..self.opt },
            ..self
        }
    }

    pub fn columns(self, columns: u16) -> Self {
        Self {
            opt: FlexOption {
                columns: Some(columns),
                ..self.opt
            },
            ..self
        }
    }

    pub fn direction(self, direction: FlexDirection) -> Self {
        Self {
            opt: FlexOption {
                direction,
                ..self.opt
            },
            ..self
        }
    }

    fn compute_columns(&self, available_width: u16) -> usize {
        let count = self.children.len();
        if count == 0 {
            return 0;
        }

        let max_cols = self
            .opt
            .columns
            .map(|c| std::cmp::min(c as usize, count))
            .unwrap_or(count)
            .max(1);

        if !self.opt.wrap {
            return max_cols;
        }

        // Search from max_cols down to 1 to find the largest column count that satisfies
        // `min_item_width`.
        for cols in (1..=max_cols).rev() {
            let cols_u16 = cols as u16;
            let gaps = self.opt.gap.saturating_mul(cols_u16.saturating_sub(1));
            let required = self.opt.min_item_width.saturating_mul(cols_u16) + gaps;
            if required <= available_width {
                return cols;
            }
        }
        1
    }

    fn widths_for_row(&self, available_width: u16, cols: usize) -> Vec<u16> {
        if cols == 0 {
            return vec![];
        }
        if cols == 1 {
            return vec![available_width];
        }

        let cols_u16 = cols as u16;
        let gaps = self.opt.gap.saturating_mul(cols_u16.saturating_sub(1));
        let usable = available_width.saturating_sub(gaps);
        let base = usable / cols_u16;
        let rem = usable % cols_u16;

        (0..cols)
            .map(|i| base + if (i as u16) < rem { 1 } else { 0 })
            .collect()
    }

    fn render_row(&self, row: &[&dyn DynModel], widths: &[u16]) -> Vec<String> {
        let child_lines: Vec<Vec<String>> = row
            .iter()
            .map(|c| c.view_string().split('\n').map(|s| s.to_string()).collect())
            .collect();

        let height = child_lines
            .iter()
            .map(|lines| lines.len())
            .max()
            .unwrap_or(0);

        let mut out = Vec::with_capacity(height);
        for line_idx in 0..height {
            let mut parts = Vec::with_capacity(row.len());
            for (col_idx, lines) in child_lines.iter().enumerate() {
                let w = *widths.get(col_idx).unwrap_or(&0);
                let raw = lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");
                let clamped = matcha::clamp_by(raw, w);
                let padded = fill_by_space(clamped, w);
                parts.push(padded);
            }
            out.push(parts.join(&" ".repeat(self.opt.gap as usize)));
        }
        out
    }
}

impl Model for Flex {
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        let mut cmds = vec![];
        let mut children: Vec<Box<dyn DynModel>> = Vec::with_capacity(self.children.len());
        for c in self.children.into_iter() {
            let (c, cmd) = c.init_box(input);
            if let Some(cmd) = cmd {
                cmds.push(cmd);
            }
            children.push(c);
        }
        let cmd = if cmds.is_empty() {
            None
        } else {
            Some(batch(cmds))
        };
        (
            Self {
                width: input.size.0,
                children,
                ..self
            },
            cmd,
        )
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        let mut cmds = vec![];
        let mut width = self.width;
        if let Some(r) = msg.downcast_ref::<ResizeEvent>() {
            width = r.0;
        }

        let mut children: Vec<Box<dyn DynModel>> = Vec::with_capacity(self.children.len());
        for c in self.children.into_iter() {
            let (c, cmd) = c.update_box(msg);
            if let Some(cmd) = cmd {
                cmds.push(cmd);
            }
            children.push(c);
        }

        let cmd = if cmds.is_empty() {
            None
        } else {
            Some(batch(cmds))
        };
        (
            Self {
                width,
                children,
                ..self
            },
            cmd,
        )
    }

    fn view(&self) -> impl Display {
        if self.children.is_empty() {
            return String::new();
        }

        let available_width = self.width;
        match self.opt.direction {
            FlexDirection::Row => {
                let cols = self.compute_columns(available_width);
                if cols == 0 {
                    return String::new();
                }
                let mut lines: Vec<String> = vec![];
                for chunk in self.children.chunks(cols) {
                    let row: Vec<&dyn DynModel> = chunk.iter().map(|c| c.as_ref()).collect();
                    let widths = self.widths_for_row(available_width, row.len());
                    lines.extend(self.render_row(&row, &widths));
                }
                lines.join("\n")
            }
            FlexDirection::Column => {
                let mut out: Vec<String> = vec![];
                for (i, child) in self.children.iter().enumerate() {
                    if i != 0 {
                        out.extend(std::iter::repeat(String::new()).take(self.opt.gap as usize));
                    }
                    let clamped_lines = child
                        .view_string()
                        .split('\n')
                        .map(|line| {
                            fill_by_space(matcha::clamp_by(line, available_width), available_width)
                        })
                        .collect::<Vec<_>>();
                    out.extend(clamped_lines);
                }
                out.join("\n")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dyn_model::boxed;

    #[derive(Clone)]
    struct Static(&'static str);
    impl Model for Static {
        fn view(&self) -> impl Display {
            self.0.to_string()
        }
    }

    #[test]
    fn picks_columns_by_min_width() {
        let flex = Flex::new(vec![
            boxed(Static("a")),
            boxed(Static("b")),
            boxed(Static("c")),
            boxed(Static("d")),
        ])
        .min_item_width(4)
        .gap(1);
        assert_eq!(flex.compute_columns(4), 1); // 4 cols needs 4*4+3=19
        assert_eq!(flex.compute_columns(9), 2); // 2 cols needs 8+1=9
        assert_eq!(flex.compute_columns(19), 4);
    }

    #[test]
    fn columns_is_max_and_still_wraps() {
        let flex = Flex::new(vec![
            boxed(Static("a")),
            boxed(Static("b")),
            boxed(Static("c")),
            boxed(Static("d")),
        ])
        .columns(4)
        .min_item_width(4)
        .gap(1);
        assert_eq!(flex.compute_columns(9), 2);
        assert_eq!(flex.compute_columns(19), 4);
    }
}
