//! `chagashi` provides higher-level terminal UI components built on top of [`matcha`].
//!
//! The crate includes reusable widgets such as:
//! - Layout (`flex`)
//! - Tabs (`tabs`)
//! - Text input / textarea (`textinput`, `textarea`)
//! - Viewport scrolling (`viewport`)
//! - Spinners (`spinner`)
//! - Borders (`border`, `borderize`)
//!
//! Most components implement [`matcha::Model`] so they can be composed.

/// Border character definitions.
pub mod border;
/// A wrapper that renders optional borders around a child model.
pub mod borderize;
mod cursor;
/// Flexbox-inspired layout container.
pub mod flex;
pub mod list;
/// Spinner widget.
pub mod spinner;
/// Tabs widget.
pub mod tabs;
pub mod textarea;
/// Single-line text input widget.
pub mod textinput;
/// A scrollable viewport wrapper.
pub mod viewport;

mod utils;

pub use flex::{Flex, FlexDirection, FlexOption};
