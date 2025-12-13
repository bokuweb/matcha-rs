use matcha::*;

use std::fmt::Display;
#[cfg(not(test))]
use std::sync::atomic::AtomicUsize;
#[cfg(not(test))]
static ID: AtomicUsize = AtomicUsize::new(1);

#[cfg(not(test))]
pub fn next_id() -> usize {
    use std::sync::atomic::Ordering;

    let id = ID.load(Ordering::Relaxed);
    ID.store(id.wrapping_add(1), Ordering::Relaxed);
    id
}

#[cfg(test)]
pub fn next_id() -> usize {
    1
}

/// Spinner is a set of frames used in animating the spinner.
#[derive(Clone, Copy, Debug)]
pub enum SpinnerType {
    Line {
        frames: [&'static str; 4],
        fps: std::time::Duration,
    },
    Dot {
        frames: [&'static str; 8],
        fps: std::time::Duration,
    },
    MiniDot {
        frames: [&'static str; 10],
        fps: std::time::Duration,
    },
    Jump {
        frames: [&'static str; 7],
        fps: std::time::Duration,
    },
    Pulse {
        frames: [&'static str; 4],
        fps: std::time::Duration,
    },
    Points {
        frames: [&'static str; 4],
        fps: std::time::Duration,
    },
    Globe {
        frames: [&'static str; 3],
        fps: std::time::Duration,
    },
    Moon {
        frames: [&'static str; 8],
        fps: std::time::Duration,
    },
    Monkey {
        frames: [&'static str; 3],
        fps: std::time::Duration,
    },
    Meter {
        frames: [&'static str; 7],
        fps: std::time::Duration,
    },
    Hamburger {
        frames: [&'static str; 4],
        fps: std::time::Duration,
    },
}

impl SpinnerType {
    pub fn line() -> Self {
        Self::Line {
            frames: ["|", "/", "-", "\\"],
            fps: std::time::Duration::from_millis(100),
        }
    }

    pub fn dot() -> Self {
        Self::Dot {
            frames: ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            fps: std::time::Duration::from_millis(100),
        }
    }

    pub fn mini_dot() -> Self {
        Self::MiniDot {
            frames: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            fps: std::time::Duration::from_millis(1000 / 12),
        }
    }

    pub fn jump() -> Self {
        Self::Jump {
            frames: ["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"],
            fps: std::time::Duration::from_millis(100),
        }
    }

    pub fn pulse() -> Self {
        Self::Pulse {
            frames: ["█", "▓", "▒", "░"],
            fps: std::time::Duration::from_millis(1000 / 8),
        }
    }

    pub fn points() -> Self {
        Self::Points {
            frames: ["∙∙∙", "●∙∙", "∙●∙", "∙∙●"],
            fps: std::time::Duration::from_millis(1000 / 7),
        }
    }

    pub fn globe() -> Self {
        Self::Globe {
            frames: ["🌍", "🌎", "🌏"],
            fps: std::time::Duration::from_millis(1000 / 4),
        }
    }

    pub fn moon() -> Self {
        Self::Moon {
            frames: ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
            fps: std::time::Duration::from_millis(1000 / 8),
        }
    }

    pub fn monkey() -> Self {
        Self::Monkey {
            frames: ["🙈", "🙉", "🙊"],
            fps: std::time::Duration::from_millis(1000 / 3),
        }
    }

    pub fn meter() -> Self {
        Self::Meter {
            frames: ["▱▱▱", "▰▱▱", "▰▰▱", "▰▰▰", "▰▰▱", "▰▱▱", "▱▱▱"],
            fps: std::time::Duration::from_millis(1000 / 7),
        }
    }

    pub fn hamburger() -> Self {
        Self::Hamburger {
            frames: ["☱", "☲", "☴", "☲"],
            fps: std::time::Duration::from_millis(1000 / 3),
        }
    }

    fn fps(&self) -> std::time::Duration {
        match self {
            Self::Line { fps, .. } => *fps,
            Self::Dot { fps, .. } => *fps,
            Self::MiniDot { fps, .. } => *fps,
            Self::Jump { fps, .. } => *fps,
            Self::Pulse { fps, .. } => *fps,
            Self::Points { fps, .. } => *fps,
            Self::Globe { fps, .. } => *fps,
            Self::Moon { fps, .. } => *fps,
            Self::Monkey { fps, .. } => *fps,
            Self::Meter { fps, .. } => *fps,
            Self::Hamburger { fps, .. } => *fps,
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Line { frames, .. } => frames.len(),
            Self::Dot { frames, .. } => frames.len(),
            Self::MiniDot { frames, .. } => frames.len(),
            Self::Jump { frames, .. } => frames.len(),
            Self::Pulse { frames, .. } => frames.len(),
            Self::Points { frames, .. } => frames.len(),
            Self::Globe { frames, .. } => frames.len(),
            Self::Moon { frames, .. } => frames.len(),
            Self::Monkey { frames, .. } => frames.len(),
            Self::Meter { frames, .. } => frames.len(),
            Self::Hamburger { frames, .. } => frames.len(),
        }
    }

    fn frames(&self) -> &[&'static str] {
        match self {
            Self::Line { frames, .. } => frames,
            Self::Dot { frames, .. } => frames,
            Self::MiniDot { frames, .. } => frames,
            Self::Jump { frames, .. } => frames,
            Self::Pulse { frames, .. } => frames,
            Self::Points { frames, .. } => frames,
            Self::Globe { frames, .. } => frames,
            Self::Moon { frames, .. } => frames,
            Self::Monkey { frames, .. } => frames,
            Self::Meter { frames, .. } => frames,
            Self::Hamburger { frames, .. } => frames,
        }
    }
}

/// Model contains the state for the spinner. Use New to create new models
/// rather than using Model as a struct literal.
pub struct Spinner {
    /// Spinner settings to use. See type Spinner.
    spinner_type: SpinnerType,
    frame: usize,
    id: usize,
    tag: usize,
    color: Option<Color>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            spinner_type: SpinnerType::line(),
            id: 0,
            frame: 0,
            tag: 0,
            color: None,
        }
    }
}

// ID returns the spinner's unique ID.
impl Spinner {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_color(self, color: Color) -> Self {
        Self {
            color: Some(color),
            ..self
        }
    }

    pub fn color(&self) -> Option<Color> {
        self.color
    }

    pub fn set_spinner_type(self, spinner: SpinnerType) -> Self {
        Self {
            spinner_type: spinner,
            ..self
        }
    }

    pub fn spinner_type(self) -> SpinnerType {
        self.spinner_type
    }

    /// New returns a model with default values.
    pub fn new(spinner_type: SpinnerType) -> Self {
        Self {
            spinner_type,
            id: next_id(),
            ..Default::default()
        }
    }

    pub fn tick(&self, tag: usize) -> Cmd {
        let id = self.id;
        tick(self.spinner_type.fps(), move || {
            Box::new(TickMsg { id, tag })
        })
    }
}

/// TickMsg indicates that the timer has ticked and we should render a frame.
pub struct TickMsg {
    pub tag: usize,
    pub id: usize,
}

impl Model for Spinner {
    /// Update is called when a message is received. Use it to inspect messages
    /// and, in response, update the model and/or send a command.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(msg) = msg.downcast_ref::<TickMsg>() {
            // If an id is set, and the id doesn't belong to this spinner, reject
            // the message.
            if msg.id > 0 && msg.id != self.id {
                return (self, None);
            }

            // If a tag is set, and it's not the one we expect, reject the message.
            // This prevents the spinner from receiving too many messages and
            // thus spinning too fast.
            if msg.tag != self.tag {
                return (self, None);
            }

            let f = if self.frame == self.spinner_type.frames().len() - 1 {
                0
            } else {
                self.frame + 1
            };

            let tag = self.tag + 1;
            return (
                Self {
                    frame: f,
                    tag,
                    ..self
                },
                Some(self.tick(tag)),
            );
        };
        (self, None)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn view(&self) -> impl Display {
        if self.frame >= self.spinner_type.len() {
            unreachable!("frame out of range");
        }
        let s = self.spinner_type.frames()[self.frame].to_string();
        if let Some(color) = self.color {
            style(s).with(color).to_string()
        } else {
            s
        }
    }
}
