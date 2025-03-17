pub mod counting_set {
    use std::{
        collections::{BTreeMap, HashMap},
        hash::Hash,
    };

    #[derive(Debug, Clone, Default)]
    pub struct CountingHashSet<T>(HashMap<T, usize>);
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

        pub fn clear(&mut self) {
            self.0.clear();
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct CountingBTreeSet<T>(BTreeMap<T, usize>);
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

        pub fn increment(&mut self, value: T) {
            *self.0.entry(value).or_insert(0) += 1;
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

        pub fn clear(&mut self) {
            self.0.clear();
        }
    }
}
