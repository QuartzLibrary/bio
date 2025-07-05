pub mod counting_set {
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::Ordering,
        collections::{BTreeMap, HashMap},
        hash::Hash,
        ops::{Add, AddAssign, Sub, SubAssign},
    };

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(transparent)]
    #[serde(bound(deserialize = "T: Deserialize<'de> + Hash + Eq"))]
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
            *self.0.entry(value).or_default() += count;
        }

        pub fn decrement(&mut self, value: &T) {
            assert!(self.try_decrement(value));
        }
        pub fn decrement_by(&mut self, value: &T, count: usize) {
            for _ in 0..count {
                self.decrement(value);
            }
        }

        /// Decrements the count by one, saturating at zero.
        /// Returns false if there were not enough items to decrement.
        pub fn try_decrement(&mut self, value: &T) -> bool {
            self.try_decrement_by(value, 1)
        }

        /// Decrements the count by the given amount, saturating at zero.
        /// Returns false if there were not enough items to decrement.
        pub fn try_decrement_by(&mut self, value: &T, count: usize) -> bool {
            if count == 0 {
                return true;
            }

            let Some(c) = self.0.get_mut(value) else {
                return false;
            };

            assert_ne!(*c, 0);

            match Ord::cmp(c, &count) {
                Ordering::Less => {
                    self.0.remove(value);
                    false
                }
                Ordering::Equal => {
                    self.0.remove(value);
                    true
                }
                Ordering::Greater => {
                    *c -= count;
                    true
                }
            }
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
    impl<T> Add for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            let mut new = self;
            for (key, value) in other.0 {
                new.increment_by(key, value);
            }
            new
        }
    }
    impl<T> AddAssign for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        fn add_assign(&mut self, other: Self) {
            for (key, value) in other.0 {
                self.increment_by(key, value);
            }
        }
    }
    impl<T> Sub for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        type Output = Self;

        fn sub(self, other: Self) -> Self {
            let mut new = self;
            for (key, value) in other.0 {
                new.decrement_by(&key, value);
            }
            new
        }
    }
    impl<T> SubAssign for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        fn sub_assign(&mut self, other: Self) {
            for (key, value) in other.0 {
                self.decrement_by(&key, value);
            }
        }
    }
    impl<T> From<Vec<(T, usize)>> for CountingHashSet<T>
    where
        T: Hash + Eq,
    {
        fn from(value: Vec<(T, usize)>) -> Self {
            value.into_iter().collect()
        }
    }
    impl<T> From<CountingHashSet<T>> for Vec<(T, usize)>
    where
        T: Hash + Eq + Clone,
    {
        fn from(value: CountingHashSet<T>) -> Self {
            value.into_iter().collect()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    #[serde(bound(deserialize = "T: Deserialize<'de> + Ord"))]
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
            *self.0.entry(value).or_default() += 1;
        }
        pub fn increment_by(&mut self, value: T, count: usize) {
            *self.0.entry(value).or_default() += count;
        }

        pub fn decrement(&mut self, value: &T) {
            assert!(self.try_decrement(value));
        }
        pub fn decrement_by(&mut self, value: &T, count: usize) {
            for _ in 0..count {
                self.decrement(value);
            }
        }

        /// Decrements the count by one, saturating at zero.
        /// Returns false if there were not enough items to decrement.
        pub fn try_decrement(&mut self, value: &T) -> bool {
            self.try_decrement_by(value, 1)
        }

        /// Decrements the count by the given amount, saturating at zero.
        /// Returns false if there were not enough items to decrement.
        pub fn try_decrement_by(&mut self, value: &T, count: usize) -> bool {
            if count == 0 {
                return true;
            }

            let Some(c) = self.0.get_mut(value) else {
                return false;
            };

            assert_ne!(*c, 0);

            match Ord::cmp(c, &count) {
                Ordering::Less => {
                    self.0.remove(value);
                    false
                }
                Ordering::Equal => {
                    self.0.remove(value);
                    true
                }
                Ordering::Greater => {
                    *c -= count;
                    true
                }
            }
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
    impl<T> Add for CountingBTreeSet<T>
    where
        T: Ord,
    {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            let mut new = self;
            for (key, value) in other.0 {
                new.increment_by(key, value);
            }
            new
        }
    }
    impl<T> AddAssign for CountingBTreeSet<T>
    where
        T: Ord,
    {
        fn add_assign(&mut self, other: Self) {
            for (key, value) in other.0 {
                self.increment_by(key, value);
            }
        }
    }
    impl<T> Sub for CountingBTreeSet<T>
    where
        T: Ord,
    {
        type Output = Self;

        fn sub(self, other: Self) -> Self {
            let mut new = self;
            for (key, value) in other.0 {
                new.decrement_by(&key, value);
            }
            new
        }
    }
    impl<T> SubAssign for CountingBTreeSet<T>
    where
        T: Ord,
    {
        fn sub_assign(&mut self, other: Self) {
            for (key, value) in other.0 {
                self.decrement_by(&key, value);
            }
        }
    }
    impl<T> From<Vec<(T, usize)>> for CountingBTreeSet<T>
    where
        T: Ord,
    {
        fn from(value: Vec<(T, usize)>) -> Self {
            value.into_iter().collect()
        }
    }
    impl<T> From<CountingBTreeSet<T>> for Vec<(T, usize)>
    where
        T: Ord + Clone,
    {
        fn from(value: CountingBTreeSet<T>) -> Self {
            value.into_iter().collect()
        }
    }
}
