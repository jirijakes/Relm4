use crate::Sender;

use crate::factory::sync::builder::FactoryBuilder;
use crate::factory::sync::handle::FactoryHandle;
use crate::factory::{CloneableFactoryComponent, FactoryComponent, FactoryView};

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::iter::FusedIterator;
use std::ops;

#[derive(Debug)]
#[must_use]
pub struct FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    inner: &'a mut FactoryHandle<C>,
}

impl<'a, C> ops::Deref for FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.inner.data.get()
    }
}

impl<'a, C> ops::DerefMut for FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.data.get_mut()
    }
}

impl<'a, C> Drop for FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    fn drop(&mut self) {
        self.inner.notifier.send(()).unwrap()
    }
}

/// A container similar to [`HashMap`] that can be used to store
/// values of type [`FactoryComponent`].
#[derive(Debug)]
pub struct FactoryHashMap<K, C: FactoryComponent, S = RandomState> {
    widget: C::ParentWidget,
    parent_sender: Sender<C::ParentInput>,
    inner: HashMap<K, FactoryHandle<C>, S>,
}

impl<K, C, S> Drop for FactoryHashMap<K, C, S>
where
    C: FactoryComponent,
{
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K, C, S> ops::Index<&K> for FactoryHashMap<K, C, S>
where
    C: FactoryComponent<Index = K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    type Output = C;

    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("Called `get` on an invalid key")
    }
}

impl<K, C> FactoryHashMap<K, C, RandomState>
where
    C: FactoryComponent,
{
    /// Creates a new [`FactoryHashMap`].
    #[must_use]
    pub fn new(widget: C::ParentWidget, parent_sender: &Sender<C::ParentInput>) -> Self {
        Self {
            widget,
            parent_sender: parent_sender.clone(),
            inner: HashMap::new(),
        }
    }
}

impl<K, C, S> FactoryHashMap<K, C, S>
where
    C: FactoryComponent,
{
    /// Creates a new [`FactoryHashMap`].
    #[must_use]
    pub fn width_hasher(
        widget: C::ParentWidget,
        parent_sender: &Sender<C::ParentInput>,
        hash_builder: S,
    ) -> Self {
        Self {
            widget,
            parent_sender: parent_sender.clone(),
            inner: HashMap::with_hasher(hash_builder),
        }
    }

    /// Returns the number of elements in the [`FactoryHashMap`].
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the [`FactoryHashMap`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Send clone of a message to all of the elements.
    pub fn broadcast(&self, msg: C::Input)
    where
        C::Input: Clone,
    {
        self.inner.values().for_each(|c| c.input.emit(msg.clone()));
    }

    /// Returns the widget all components are attached to.
    pub const fn widget(&self) -> &C::ParentWidget {
        &self.widget
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &C)> + ExactSizeIterator + FusedIterator {
        self.inner.iter().map(|(k, c)| (k, c.data.get()))
    }

    /// Returns an iterator over the factory components.
    pub fn values(&self) -> impl Iterator<Item = &C> + ExactSizeIterator + FusedIterator {
        self.inner.values().map(|c| c.data.get())
    }

    /// Returns an iterator over the keys of the hash map.
    pub fn keys(&self) -> impl Iterator<Item = &K> + ExactSizeIterator + FusedIterator {
        self.inner.keys()
    }

    /// Clears the map, removing all factory components.
    pub fn clear(&mut self) {
        for (_, handle) in self.inner.drain() {
            self.widget.factory_remove(&handle.returned_widget);
        }
    }
}

impl<K, C> FactoryHashMap<K, C, RandomState>
where
    C: FactoryComponent<Index = K>,
    K: Hash + Eq,
{
    /// Creates a [`FactoryHashMap`] from a [`Vec`].
    pub fn from_vec(
        component_vec: Vec<(K, C::Init)>,
        widget: C::ParentWidget,
        parent_sender: &Sender<C::ParentInput>,
    ) -> Self {
        let mut output = Self::new(widget, parent_sender);
        for (key, init) in component_vec {
            output.insert(key, init);
        }
        output
    }
}

impl<K, C, S> FactoryHashMap<K, C, S>
where
    C: FactoryComponent<Index = K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Send a mage to one of the elements.
    pub fn send(&self, key: &K, msg: C::Input) {
        self.inner[key].input.emit(msg);
    }

    /// Tries to get an immutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `key` is invalid.
    pub fn get(&self, key: &K) -> Option<&C> {
        self.inner.get(key).map(|c| c.data.get())
    }

    /// Tries to get a mutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `key` is invalid.
    pub fn get_mut(&mut self, key: &K) -> Option<FactoryElementGuard<'_, C>> {
        self.inner
            .get_mut(key)
            .map(|c| FactoryElementGuard { inner: c })
    }

    /// Inserts a new factory component into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though; this matters for types that can be == without being identical. See the module-level documentation for more.
    pub fn insert(&mut self, key: K, init: C::Init) -> Option<C> {
        let existing = self.remove(&key);

        let builder = FactoryBuilder::new(&key, init);

        let position = C::position(&builder.data, &key);
        let returned_widget = self
            .widget
            .factory_append(builder.root_widget.clone(), &position);

        let component = builder.launch(
            &key,
            returned_widget,
            &self.parent_sender,
            C::forward_to_parent,
        );

        assert!(self.inner.insert(key, component).is_none());

        existing
    }

    /// Removes a key from the map, returning the factory component at the key if the key was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<C> {
        if let Some(handle) = self.inner.remove(key) {
            self.widget.factory_remove(&handle.returned_widget);
            Some(handle.data.into_inner())
        } else {
            None
        }
    }
}

/// Implements the Clone Trait for [`FactoryHashMap`] if the component implements [`CloneableFactoryComponent`].
impl<K, C> Clone for FactoryHashMap<K, C, RandomState>
where
    C: CloneableFactoryComponent,
    K: Clone + Hash + Eq,
    C: FactoryComponent<Index = K>,
{
    fn clone(&self) -> Self {
        // Create a new, empty FactoryHashMap.
        let mut clone = FactoryHashMap::new(self.widget.clone(), &self.parent_sender.clone());
        // Iterate over the items in the original FactoryHashMap.
        for (k, item) in self.iter() {
            // Clone each item and push it onto the new FactoryHashMap.
            let init = C::get_init(item);
            clone.insert(k.clone(), init);
        }
        // Return the new, cloned FactoryHashMap.
        clone
    }
}
