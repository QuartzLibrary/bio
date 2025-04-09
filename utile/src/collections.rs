pub mod counting_set {
    use std::{
        collections::{BTreeMap, HashMap},
        hash::Hash,
    };

    #[derive(Debug, Clone)]
    pub struct CountingHashSet<T>(HashMap<T, usize>);
    impl<T> Default for CountingHashSet<T> {
        fn default() -> Self {
            Self(Default::default())
        }
    }
    impl<T: Hash + Eq> CountingHashSet<T> {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub fn total_count(&self) -> usize {
            self.0.values().sum()
        }

        pub fn increment(&mut self, value: T) {
            *self.0.entry(value).or_default() += 1;
        }
        pub fn increment_by(&mut self, value: T, count: usize) {
            *self.0.entry(value).or_insert(0) += count;
        }
        pub fn decrement(&mut self, value: &T) {
            assert!(self.try_decrement(value));
        }
        pub fn try_decrement(&mut self, value: &T) -> bool {
            let Some(c) = self.0.get_mut(value) else {
                return false;
            };

            *c -= 1;

            if *c == 0 {
                self.0.remove(value);
            }

            true
        }
        pub fn remove(&mut self, value: &T) -> Option<usize> {
            self.0.remove(value)
        }

        pub fn count(&self, value: &T) -> usize {
            self.0.get(value).copied().unwrap_or(0)
        }
        pub fn contains(&self, value: &T) -> bool {
            self.0.contains_key(value)
        }

        pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
            self.into_iter()
        }

        pub fn clear(&mut self) {
            self.0.clear();
        }
    }
    impl<T> FromIterator<T> for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut set = Self::new();
            for value in iter {
                set.increment(value);
            }
            set
        }
    }
    impl<T> FromIterator<(T, usize)> for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
            let mut set = Self::new();
            for (value, count) in iter {
                set.increment_by(value, count);
            }
            set
        }
    }
    impl<T> IntoIterator for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        type Item = (T, usize);
        type IntoIter = std::collections::hash_map::IntoIter<T, usize>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
    impl<'a, T> IntoIterator for &'a CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        type Item = (&'a T, &'a usize);
        type IntoIter = std::collections::hash_map::Iter<'a, T, usize>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct CountingBTreeSet<T>(BTreeMap<T, usize>);
    impl<T> Default for CountingBTreeSet<T> {
        fn default() -> Self {
            Self(Default::default())
        }
    }
    impl<T: Ord> CountingBTreeSet<T> {
        pub fn new() -> Self {
            Self(BTreeMap::new())
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub fn total_count(&self) -> usize {
            self.0.values().sum()
        }

        pub fn first(&self) -> Option<&T> {
            self.0.first_key_value().map(|(k, _v)| k)
        }
        pub fn last(&self) -> Option<&T> {
            self.0.last_key_value().map(|(k, _v)| k)
        }

        pub fn increment(&mut self, value: T) {
            *self.0.entry(value).or_insert(0) += 1;
        }
        pub fn increment_by(&mut self, value: T, count: usize) {
            *self.0.entry(value).or_insert(0) += count;
        }
        pub fn decrement(&mut self, value: &T) {
            assert!(self.try_decrement(value));
        }
        pub fn try_decrement(&mut self, value: &T) -> bool {
            let Some(c) = self.0.get_mut(value) else {
                return false;
            };

            *c -= 1;

            if *c == 0 {
                self.0.remove(value);
            }

            true
        }
        pub fn remove(&mut self, value: &T) -> Option<usize> {
            self.0.remove(value)
        }

        pub fn count(&self, value: &T) -> usize {
            self.0.get(value).copied().unwrap_or(0)
        }
        pub fn contains(&self, value: &T) -> bool {
            self.0.contains_key(value)
        }

        pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
            self.into_iter()
        }

        pub fn clear(&mut self) {
            self.0.clear();
        }
    }
    impl<T> FromIterator<T> for CountingBTreeSet<T>
    where
        T: Ord,
    {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut set = Self::new();
            for value in iter {
                set.increment(value);
            }
            set
        }
    }
    impl<T> FromIterator<(T, usize)> for CountingBTreeSet<T>
    where
        T: Ord,
    {
        fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
            let mut set = Self::new();
            for (value, count) in iter {
                set.increment_by(value, count);
            }
            set
        }
    }
    impl<T> IntoIterator for CountingBTreeSet<T>
    where
        T: Ord,
    {
        type Item = (T, usize);
        type IntoIter = std::collections::btree_map::IntoIter<T, usize>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
    impl<'a, T> IntoIterator for &'a CountingBTreeSet<T>
    where
        T: Ord,
    {
        type Item = (&'a T, &'a usize);
        type IntoIter = std::collections::btree_map::Iter<'a, T, usize>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }
}
