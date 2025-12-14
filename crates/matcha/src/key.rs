use {
    ::crossterm::event::KeyEvent,
    crokey::KeyCombination,
    std::{collections::HashMap, ops::Deref},
};

/// A structure for storing and retrieving bindings between [`Key`] and arbitrary data.
///
/// This is especially useful for setting up configuration or user-defined key mappings
/// to certain functionalities within an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindings<T>(pub HashMap<Key, T>);

impl<T> KeyBindings<T> {
    /// Creates a new [`KeyBindings`] instance from a [`HashMap`] of [`KeyCombination`] to `T`.
    ///
    /// This constructor transforms each [`KeyCombination`] into a [`Key`] by wrapping it.
    ///
    /// # Parameters
    ///
    /// * `bindings` - A [`HashMap`] where the key is a [`KeyCombination`] and the value is of type `T`.
    ///
    /// # Returns
    ///
    /// A new instance of [`KeyBindings<T>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use matcha::*;
    /// use chagashi::textarea::*;
    ///
    /// let bindings = [
    ///     (key!(ctrl - b), TextareaKeys::MoveLeft),
    ///     (key!(left), TextareaKeys::MoveLeft),
    ///     (key!(ctrl - f), TextareaKeys::MoveRight),
    ///     (key!(right), TextareaKeys::MoveRight),
    ///     (key!(ctrl - p), TextareaKeys::MoveUp),
    ///     (key!(up), TextareaKeys::MoveUp),
    ///     (key!(ctrl - n), TextareaKeys::MoveDown),
    ///     (key!(down), TextareaKeys::MoveDown),
    ///     (key!(enter), TextareaKeys::InsertNewline),
    ///     (key!(ctrl - m), TextareaKeys::InsertNewline),
    ///     (key!(backspace), TextareaKeys::DeleteBack),
    ///     (key!(ctrl - h), TextareaKeys::DeleteBack),
    ///     (key!(delete), TextareaKeys::DeleteForward),
    ///     (key!(ctrl - d), TextareaKeys::DeleteForward),
    /// ]
    /// .into_iter()
    /// .collect();
    /// KeyBindings::new(bindings);
    /// ```
    pub fn new(bindings: HashMap<KeyCombination, T>) -> Self {
        Self(bindings.into_iter().map(|k| (Key(k.0), k.1)).collect())
    }

    /// Get a binding by key.
    pub fn get(&self, k: Key) -> Option<&T> {
        self.0.get(&k)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
/// A thin wrapper around [`KeyCombination`].
///
/// This type is used as the key for [`KeyBindings`].
pub struct Key(pub KeyCombination);

impl From<&KeyEvent> for Key {
    fn from(value: &KeyEvent) -> Self {
        let e = crokey::crossterm::event::KeyEvent {
            code: value.code,
            modifiers: value.modifiers,
            kind: value.kind,
            state: value.state,
        };
        Self(crokey::KeyCombination::from(e))
    }
}

impl Deref for Key {
    type Target = KeyCombination;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<KeyEvent> for Key {
    fn from(value: KeyEvent) -> Self {
        Self::from(&value)
    }
}

impl Key {
    /// Return true if this key matches the given combination.
    pub fn matches(&self, k: KeyCombination) -> bool {
        self.0 == k
    }

    /// Return the underlying [`KeyCombination`].
    pub fn combination(&self) -> KeyCombination {
        self.0
    }
}
