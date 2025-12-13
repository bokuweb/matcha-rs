use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

/// A map that stores arbitrary types using [`TypeId`] as a key.
/// Values are stored as a boxed [`Any`] wrapped in an [`Arc`].
type AnyMap = HashMap<TypeId, Arc<Box<dyn Any + Send + Sync>>>;

/// A container that allows storing and retrieving extension data in a type-safe manner.
/// Internally, it uses [`AnyMap`] to manage the data.
#[derive(Debug, Default, Clone)]
pub struct Extensions(pub AnyMap);

impl Extensions {
    /// Creates a new, empty [`Extensions`].
    ///
    /// # Examples
    ///
    /// ```
    /// use matcha::Extensions;
    ///
    /// let extensions = Extensions::new();
    /// assert!(extensions.0.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a value of arbitrary type `T` into the container.
    ///
    /// `T` must be `'static + Send + Sync`. If a value of the same type `T` already exists,
    /// it will be overwritten.
    ///
    /// # Parameters
    ///
    /// * `item` - The value to be inserted.
    ///
    /// # Examples
    ///
    /// ```
    /// use matcha::Extensions;
    ///
    /// let mut extensions = Extensions::new();
    /// extensions.insert(42u32);
    /// extensions.insert("Hello, Rust!");
    /// ```
    pub fn insert<T>(&mut self, item: T)
    where
        T: 'static + Send + Sync,
    {
        let key = item.type_id().to_owned();
        let boxed: Arc<Box<dyn Any + Send + Sync>> = Arc::new(Box::new(item));
        self.0.insert(key, boxed);
    }

    /// Retrieves a reference to a value of type `T` if it exists in the container.
    /// Returns `None` if the type doesn't match or if it hasn't been stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use matcha::Extensions;
    ///
    /// let mut extensions = Extensions::new();
    /// extensions.insert(42u32);
    /// if let Some(value) = extensions.get::<u32>() {
    ///     assert_eq!(*value, 42);
    /// } else {
    ///     panic!("`u32` value not found.");
    /// }
    /// ```
    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        let key = TypeId::of::<T>();
        let item = self.0.get(&key);

        item.and_then(|any| any.downcast_ref())
    }

    /// Retrieves a reference to a value of type `T` and panics if it does not exist.
    ///
    /// # Panics
    ///
    /// * Panics if a value of type `T` is not stored in the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use matcha::Extensions;
    ///
    /// let mut extensions = Extensions::new();
    /// extensions.insert(42u32);
    ///
    /// // This will panic if a `u32` value does not exist.
    /// let value = extensions.get_unchecked::<u32>();
    /// assert_eq!(*value, 42);
    /// ```
    pub fn get_unchecked<T>(&self) -> &T
    where
        T: 'static,
    {
        self.get::<T>().unwrap()
    }
}
