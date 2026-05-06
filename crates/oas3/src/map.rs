//! Order-preserving key-value map.

use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    hash::Hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// A map that preserves insertion order.
#[derive(Debug, Clone)]
pub struct Map<K, V> {
    inner: indexmap::IndexMap<K, V>,
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> PartialEq for Map<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<K, V> Eq for Map<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

impl<K, V> Map<K, V> {
    /// Creates an empty map.
    pub fn new() -> Self {
        Self {
            inner: indexmap::IndexMap::new(),
        }
    }

    /// Creates an empty map with capacity for at least `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: indexmap::IndexMap::with_capacity(capacity),
        }
    }

    /// Returns the number of entries in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes all entries from the map.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an iterator over key-value pairs in insertion order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.inner.iter(),
        }
    }

    /// Returns a mutable iterator over key-value pairs in insertion order.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Returns an iterator over keys in insertion order.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            inner: self.inner.keys(),
        }
    }

    /// Returns an iterator over values in insertion order.
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            inner: self.inner.values(),
        }
    }

    /// Returns a mutable iterator over values in insertion order.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            inner: self.inner.values_mut(),
        }
    }
}

impl<K, V> Map<K, V>
where
    K: Eq + Hash,
{
    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.inner.get_mut(key)
    }

    /// Returns `true` if the map contains a value for the key.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.inner.contains_key(key)
    }

    /// Removes a key from the map, preserving the relative order of remaining entries.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.inner.shift_remove(key)
    }
}

impl<K, V> Serialize for Map<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_map(self.inner.iter())
    }
}

impl<'de, K, V> Deserialize<'de> for Map<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MapVisitor<K, V> {
            marker: PhantomData<fn() -> Map<K, V>>,
        }

        impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
        where
            K: Deserialize<'de> + Eq + Hash,
            V: Deserialize<'de>,
        {
            type Value = Map<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut map = Map::with_capacity(access.size_hint().unwrap_or(0));

                while let Some((key, value)) = access.next_entry()? {
                    map.insert(key, value);
                }

                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor {
            marker: PhantomData,
        })
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<K, V> Extend<(K, V)> for Map<K, V>
where
    K: Eq + Hash,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        self.inner.extend(iter);
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for Map<K, V>
where
    K: Eq + Hash,
{
    fn from(value: [(K, V); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<K, V> From<BTreeMap<K, V>> for Map<K, V>
where
    K: Eq + Hash,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        value.into_iter().collect()
    }
}

impl<K, V> From<Map<K, V>> for BTreeMap<K, V>
where
    K: Ord,
{
    fn from(value: Map<K, V>) -> Self {
        value.into_iter().collect()
    }
}

impl<K, V> From<HashMap<K, V>> for Map<K, V>
where
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        value.into_iter().collect()
    }
}

impl<K, V> From<Map<K, V>> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: Map<K, V>) -> Self {
        value.into_iter().collect()
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type IntoIter = Iter<'a, K, V>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
    type IntoIter = IterMut<'a, K, V>;
    type Item = (&'a K, &'a mut V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, Q> Index<&Q> for Map<K, V>
where
    K: Borrow<Q> + Eq + Hash,
    Q: Eq + Hash + ?Sized,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        Index::index(&self.inner, index)
    }
}

impl<K, V, Q> IndexMut<&Q> for Map<K, V>
where
    K: Borrow<Q> + Eq + Hash,
    Q: Eq + Hash + ?Sized,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.inner, index)
    }
}

/// An owning iterator over map entries.
#[derive(Debug)]
pub struct IntoIter<K, V> {
    inner: indexmap::map::IntoIter<K, V>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}
impl<K, V> std::iter::FusedIterator for IntoIter<K, V> {}

/// An iterator over map entries.
#[derive(Debug, Clone)]
pub struct Iter<'a, K, V> {
    inner: indexmap::map::Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K, V> std::iter::FusedIterator for Iter<'_, K, V> {}

/// A mutable iterator over map entries.
#[derive(Debug)]
pub struct IterMut<'a, K, V> {
    inner: indexmap::map::IterMut<'a, K, V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {}
impl<K, V> std::iter::FusedIterator for IterMut<'_, K, V> {}

/// An iterator over map keys.
#[derive(Debug, Clone)]
pub struct Keys<'a, K, V> {
    inner: indexmap::map::Keys<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {}
impl<K, V> std::iter::FusedIterator for Keys<'_, K, V> {}

/// An iterator over map values.
#[derive(Debug, Clone)]
pub struct Values<'a, K, V> {
    inner: indexmap::map::Values<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {}
impl<K, V> std::iter::FusedIterator for Values<'_, K, V> {}

/// A mutable iterator over map values.
#[derive(Debug)]
pub struct ValuesMut<'a, K, V> {
    inner: indexmap::map::ValuesMut<'a, K, V>,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {}
impl<K, V> std::iter::FusedIterator for ValuesMut<'_, K, V> {}
