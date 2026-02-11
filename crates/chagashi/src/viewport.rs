use std::fmt::Display;

use matcha::*;

/// KeyMap defines the keybindings for the viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportKeys {
    /// Page down.
    PageDown,
    /// Page up.
    PageUp,
    /// Down one line.
    Down,
    /// Up one line.
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Default keybinding set for [`Viewport`].
pub struct Keybindings(matcha::KeyBindings<ViewportKeys>);

/// Emit a message that indicates an item/line has been selected.
///
/// This is typically sent by the viewport when selection mode is enabled.
pub fn selected(index: u16) -> Msg {
    Box::new(ViewportOnSelectMsg { index })
}

#[derive(Debug)]
/// Message emitted when the selection changes.
pub struct ViewportOnSelectMsg {
    /// Selected index (0-based).
    pub index: u16,
}

impl ViewportOnSelectMsg {
    /// Create a new `ViewportOnSelectMsg`.
    pub fn new(index: u16) -> Self {
        Self { index }
    }
}

/// Request selecting the given index.
pub fn select(index: u16) -> Msg {
    Box::new(ViewportSelectMsg { index })
}

#[derive(Debug)]
/// Message requesting the viewport to select an index.
pub struct ViewportSelectMsg {
    /// Index to select (0-based).
    pub index: u16,
}

impl ViewportSelectMsg {
    /// Create a new `ViewportSelectMsg`.
    pub fn new(index: u16) -> Self {
        Self { index }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        let bindings = [
            (key!(ctrl - n), ViewportKeys::Down),
            (key!(down), ViewportKeys::Down),
            (key!(ctrl - p), ViewportKeys::Up),
            (key!(up), ViewportKeys::Up),
            (key!(ctrl - v), ViewportKeys::PageDown),
            (key!(alt - v), ViewportKeys::PageUp),
        ]
        .into_iter()
        .collect();
        Keybindings(KeyBindings::new(bindings))
    }
}

/// the matcha model for this viewport element.
///
/// `Viewport` renders a child model and provides vertical scrolling. It can optionally
/// run in selection mode to highlight a line and emit selection messages.
pub struct Viewport<M> {
    width: u16,
    height: u16,
    key_bindings: Keybindings,
    /// offset_y is the vertical scroll position.
    offset_y: u16,
    wrap: bool,
    // selection
    selection: bool,
    selection_y: u16,
    selection_fg: Color,
    selection_bg: Color,
    child: M,
}

#[derive(Debug)]
/// Configuration for [`Viewport`].
pub struct ViewportOption {
    /// enable wrap mode.
    pub wrap: bool,
    /// enable selection mode.
    pub selection: bool,
    /// selection foreground color.
    pub selection_fg: Color,
    /// selection background color.
    pub selection_bg: Color,
}

impl Default for ViewportOption {
    fn default() -> Self {
        Self {
            wrap: false,
            selection: false,
            selection_fg: Color::Black,
            selection_bg: Color::Yellow,
        }
    }
}

impl<M: Model> Viewport<M> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Create a new viewport around `child` with a fixed `(width, height)` and options.
    pub fn new(child: M, size: (u16, u16), opt: ViewportOption) -> Self {
        Self {
            width: size.0,
            height: size.1,
            key_bindings: Keybindings::default(),
            offset_y: 0,
            wrap: opt.wrap,
            // selection config
            selection_y: 0,
            selection: opt.selection,
            selection_fg: opt.selection_fg,
            selection_bg: opt.selection_bg,
            child,
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Update the viewport size.
    pub fn update_size(self, size: (u16, u16)) -> Self {
        Self {
            width: size.0,
            height: size.1,
            ..self
        }
    }

    /// at_bottom returns whether or not the viewport is at the very top position.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn at_bottom(&self) -> bool {
        self.offset_y >= self.max_y_offset()
    }

    /// sets the viewport to the top position.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    ///
    /// This resets the vertical scroll offset.
    pub fn move_to_top(self) -> Self {
        Self {
            offset_y: 0,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Scroll up by one line (or move selection up in selection mode).
    pub fn move_up(self) -> Self {
        if self.selection {
            if self.selection_y <= self.offset_y {
                let offset_y = std::cmp::min(
                    self.offset_y.saturating_sub(self.height / 2),
                    self.max_y_offset(),
                );
                return Self {
                    offset_y,
                    selection_y: self.selection_y.saturating_sub(1),
                    ..self
                };
            } else {
                return Self {
                    selection_y: self.selection_y.saturating_sub(1),
                    ..self
                };
            }
        }

        Self {
            offset_y: self.offset_y.saturating_sub(1),
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Scroll down by one line (or move selection down in selection mode).
    pub fn move_down(self) -> Self {
        if self.selection {
            if self.selection_y >= (self.offset_y + self.height).saturating_sub(1) {
                let offset_y = std::cmp::min(self.offset_y + self.height / 2, self.max_y_offset());
                return Self {
                    offset_y,
                    selection_y: std::cmp::min(
                        self.selection_y + 1,
                        self.content_len().saturating_sub(1),
                    ),
                    ..self
                };
            } else {
                return Self {
                    selection_y: std::cmp::min(
                        self.selection_y + 1,
                        self.content_len().saturating_sub(1),
                    ),
                    ..self
                };
            }
        }

        if self.at_bottom() {
            return self;
        }

        Self {
            offset_y: self.offset_y + 1,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Scroll up by one page.
    pub fn page_up(self) -> Self {
        let y = self.offset_y.saturating_sub(self.height);
        Self {
            offset_y: y,
            selection_y: y,
            ..self
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    /// Scroll down by one page.
    pub fn page_down(self) -> Self {
        if self.offset_y + self.height >= self.content_len().saturating_sub(1) {
            return self;
        }

        let y = std::cmp::min(
            self.offset_y + self.height,
            self.content_len().saturating_sub(1),
        );
        Self {
            offset_y: y,
            selection_y: y,
            ..self
        }
    }

    /// Renders the child view into padded lines, applying wrapping and selection styling.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn lines(&self) -> Vec<String> {
        let child = format!("{}", self.child.view());
        child
            .split('\n')
            .enumerate()
            .flat_map(|(i, line)| self.render_segments(line, self.is_selected_line(i)))
            .collect()
    }

    /// Returns true if the 0-based index corresponds to the currently selected line.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn is_selected_line(&self, index: usize) -> bool {
        self.selection && index == self.selection_y as usize
    }

    /// Splits a line into renderable segments based on the wrap configuration.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn render_segments(&self, line: &str, is_selected: bool) -> Vec<String> {
        if self.wrap {
            self.render_wrapped_segments(line, is_selected)
        } else {
            vec![self.render_single_segment(line, is_selected)]
        }
    }

    /// Wraps a line at the viewport width and renders each resulting segment.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn render_wrapped_segments(&self, line: &str, is_selected: bool) -> Vec<String> {
        matcha::wrap(line, self.width)
            .into_iter()
            .map(|segment| self.render_wrapped_segment(&segment, is_selected))
            .collect()
    }

    /// Pads and styles a wrapped segment.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn render_wrapped_segment(&self, segment: &str, is_selected: bool) -> String {
        let padded = self.pad_to_width(segment);
        if is_selected {
            self.highlight_selection(padded)
        } else {
            style(padded).to_string()
        }
    }

    /// Pads and styles a single unwrapped segment.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn render_single_segment(&self, segment: &str, is_selected: bool) -> String {
        let padded = self.pad_to_width(segment);
        if is_selected {
            self.highlight_selection(padded)
        } else {
            padded
        }
    }

    /// Right-pads the segment with spaces to match the viewport width.
    fn pad_to_width(&self, segment: &str) -> String {
        let len = segment.len() as u16;
        let padding = self.width.saturating_sub(len) as usize;
        if padding == 0 {
            return segment.to_string();
        }
        let mut result = String::with_capacity(segment.len() + padding);
        result.push_str(segment);
        result.push_str(&" ".repeat(padding));
        result
    }

    /// Applies the configured selection colors to the given text.
    fn highlight_selection(&self, text: String) -> String {
        style(text)
            .with(self.selection_fg)
            .on(self.selection_bg)
            .to_string()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn content_len(&self) -> u16 {
        self.lines().len() as u16
    }

    /// max_y_offset returns the maximum possible value of the y-offset based on the
    /// viewport's content and set height.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn max_y_offset(&self) -> u16 {
        std::cmp::max(0, self.content_len().saturating_sub(self.height))
    }

    /// sets the viewport to the bottom position.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    ///
    /// This sets the vertical scroll offset to the maximum.
    pub fn move_to_bottom(self) -> Self {
        Self {
            offset_y: self.max_y_offset(),
            ..self
        }
    }

    /// content set the pager's text content. For high performance rendering the
    /// Sync command should also be called.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    ///
    /// If the current offset is out of range after the update, it is clamped to the bottom.
    pub fn update_content(self, child: M) -> Self {
        let s = Self { child, ..self };
        if s.offset_y > s.content_len().saturating_sub(1) {
            Self {
                ..s.move_to_bottom()
            }
        } else {
            s
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn visible_lines(&self) -> Vec<String> {
        let top = std::cmp::max(0, self.offset_y) as usize;
        let over = self.content_len() - self.offset_y > self.height;
        let bottom = if over {
            (self.offset_y + self.height) as usize
        } else {
            self.content_len() as usize
        };
        let mut lines: Vec<String> = self.lines()[top..bottom]
            .iter()
            .map(|line| matcha::clamp_by(line, self.width).replace('\r', ""))
            .collect();

        // if not overed, fill with \n to keep height.
        if !over {
            lines.extend(
                std::iter::repeat(String::new())
                    .take(self.height.saturating_sub(self.content_len()) as usize),
            );
        }
        lines
    }
}

impl<M: Model> Model for Viewport<M> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn init(self, input: &InitInput) -> (Self, Option<Cmd>) {
        let (child, cmd) = self.child.init(input);
        let mut cmds = vec![];
        if let Some(cmd) = cmd {
            cmds.push(cmd);
        }
        if self.selection {
            #[allow(unused)]
            let m = Box::new(ViewportOnSelectMsg::new(self.selection_y));
            cmds.push(Cmd::sync(Box::new(move || m)));
        }
        let cmd = if cmds.is_empty() {
            None
        } else {
            Some(batch(cmds))
        };
        (
            Self {
                width: input.size.0,
                height: input.size.1,
                child,
                ..self
            },
            cmd,
        )
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        let (new_child, child_cmd) = self.child.update(msg);
        let mut commands = vec![];
        if let Some(c) = child_cmd {
            commands.push(c);
        }
        let old_selection_y = self.selection_y;
        let new_self = Self {
            child: new_child,
            ..self
        };
        let (new_self, cmd): (Self, Option<Cmd>) = if let Some(event) = msg.downcast_ref::<KeyEvent>()
        {
            let key = new_self.key_bindings.0.get(matcha::Key::from(event));
            let new_self = match key {
                Some(ViewportKeys::Down) => new_self.move_down(),
                Some(ViewportKeys::Up) => new_self.move_up(),
                Some(ViewportKeys::PageDown) => new_self.page_down(),
                Some(ViewportKeys::PageUp) => new_self.page_up(),
                _ => new_self,
            };

            #[cfg(feature = "tracing")]
            tracing::trace!("selection_y = {}", old_selection_y);

            if new_self.selection && old_selection_y != new_self.selection_y {
                let index = new_self.selection_y;
                let cmd = Cmd::sync(Box::new(move || Box::new(ViewportOnSelectMsg { index })));
                (new_self, Some(cmd))
            } else {
                (new_self, None)
            }
        } else {
            (new_self, None)
        };
        if let Some(c) = cmd {
            commands.push(c);
        }
        let cmd = if commands.is_empty() {
            None
        } else {
            Some(batch(commands))
        };
        (new_self, cmd)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn view(&self) -> impl Display {
        let s: String = self.visible_lines().join("\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matcha::{style, Color};

    #[derive(Clone)]
    struct StaticModel(&'static str);

    impl Model for StaticModel {
        fn view(&self) -> impl Display {
            self.0.to_string()
        }
    }

    fn build_viewport(
        opt: ViewportOption,
        view: &'static str,
        size: (u16, u16),
    ) -> Viewport<StaticModel> {
        Viewport::new(StaticModel(view), size, opt)
    }

    #[test]
    fn lines_pad_segments_to_width() {
        let viewport = build_viewport(ViewportOption::default(), "abc", (6, 1));
        assert_eq!(viewport.lines(), vec!["abc   ".to_string()]);
    }

    #[test]
    fn lines_wrap_when_enabled() {
        let opt = ViewportOption {
            wrap: true,
            ..ViewportOption::default()
        };
        let viewport = build_viewport(opt, "abcdef", (4, 2));
        assert_eq!(
            viewport.lines(),
            vec!["abcd".to_string(), "ef  ".to_string()]
        );
    }

    #[test]
    fn lines_highlight_selected_line() {
        let selection_fg = Color::White;
        let selection_bg = Color::Blue;
        let opt = ViewportOption {
            selection: true,
            selection_fg,
            selection_bg,
            ..ViewportOption::default()
        };
        let viewport = build_viewport(opt, "first\nsecond", (6, 2)).move_down();
        let lines = viewport.lines();

        assert_eq!(lines[0], "first ");
        let expected = style("second".to_string())
            .with(selection_fg)
            .on(selection_bg)
            .to_string();
        assert_eq!(lines[1], expected);
    }

    #[test]
    fn update_does_not_emit_select_msg_when_selection_disabled() {
        let viewport = build_viewport(ViewportOption::default(), "a\nb\nc", (3, 2));
        let key_event: Msg = Box::new(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        let (_, cmd) = viewport.update(&key_event);
        assert!(cmd.is_none());
    }
}
