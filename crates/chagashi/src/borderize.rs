use matcha::{fill_by_space, remove_escape_sequences, style, Color, Model, Stylize};
use std::fmt::Display;
use unicode_width::UnicodeWidthStr;

use crate::border::Border;

#[derive(Debug, Default)]
pub struct BorderOption {
    pub show: bool,
    pub color: Option<Color>,
}

pub struct Borderize<M> {
    pub top: BorderOption,
    pub right: BorderOption,
    pub bottom: BorderOption,
    pub left: BorderOption,
    pub width: Option<u16>,
    pub child: M,
}

impl<M: Model> Borderize<M> {
    pub fn new(child: M) -> Self {
        Self {
            top: BorderOption::default(),
            right: BorderOption::default(),
            bottom: BorderOption::default(),
            left: BorderOption::default(),
            child,
            width: None,
        }
    }

    pub fn width(self, w: u16) -> Self {
        Self {
            width: Some(w),
            ..self
        }
    }

    pub fn top(self, b: BorderOption) -> Self {
        Self { top: b, ..self }
    }

    pub fn right(self, b: BorderOption) -> Self {
        Self { right: b, ..self }
    }

    pub fn bottom(self, b: BorderOption) -> Self {
        Self { bottom: b, ..self }
    }

    pub fn left(self, b: BorderOption) -> Self {
        Self { left: b, ..self }
    }
}

impl<M: Model> Model for Borderize<M> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn init(self, input: &matcha::InitInput) -> (Self, Option<matcha::Cmd>) {
        let (child, cmd) = self.child.init(input);
        (Self { child, ..self }, cmd)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn update(self, msg: &matcha::Msg) -> (Self, Option<matcha::Cmd>) {
        let (child, cmd) = self.child.update(msg);
        (Self { child, ..self }, cmd)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn view(&self) -> impl Display {
        let c = self.child.view().to_string();
        let lines: Vec<String> = c.split('\n').map(|c| c.to_string()).collect();
        let w = self.width.unwrap_or_else(|| {
            lines
                .iter()
                .map(|line| remove_escape_sequences(line).width())
                .max()
                .unwrap_or_default() as u16
        });

        let b = Border::default();

        let mut lines: Vec<String> = lines
            .into_iter()
            .map(|line| {
                let left: String = if self.left.show {
                    if let Some(c) = self.left.color {
                        style(&b.left).with(c).to_string()
                    } else {
                        b.left.to_string()
                    }
                } else {
                    "".to_string()
                };
                let right: String = if self.right.show {
                    self.right
                        .color
                        .iter()
                        .cloned()
                        .map(|c| style(&b.right).with(c).to_string())
                        .collect()
                } else {
                    "".to_string()
                };
                let s = format!("{}{}{}", left, fill_by_space(line, w), right);
                // self.top
                //     .color
                //     .iter()
                //     .cloned()
                //     .map(|c| style(s.clone()).with(c).to_string())
                //     .collect()
                s
            })
            .collect();

        if self.top.show {
            let b = format!("{}{}{}", b.top_left, b.top.repeat(w as usize), b.top_right);
            // let b = self
            //     .top
            //     .color
            //     .iter()
            //     .cloned()
            //     .map(|c| style(&b).with(c).to_string())
            //     .collect();
            lines.insert(0, b);
        }

        if self.bottom.show {
            lines.push(format!(
                "{}{}{}",
                b.bottom_left,
                b.bottom.repeat(w as usize),
                b.bottom_right
            ));
        }
        lines.join("\n")
    }
}
