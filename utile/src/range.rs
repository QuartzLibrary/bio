use std::{
    iter::Step,
    ops::{Add, Range, RangeInclusive, Sub},
};

/// Some helper methods for ranges.
///
/// 'Malformed' (r.end < r.start) ranges are assumed to be empty and 'based' on their start position.
/// (That is, a malformed range ~ `r.start..r.start`.)
pub trait RangeExt: Sized {
    type Item;

    /// Normalizes 'malformed' (r.end < r.start) ranges.
    /// Note that it's not always possible to normalize ranges to be 'well formed'.
    fn normalize(self) -> Self;

    /// Would the slices returned by indexing these ranges overlap?
    ///
    /// Note: might still overlap even if the ranges are empty (for example `1..1` overlaps `0..3` and `1..1)
    ///
    /// You can use `range.intersection(other).is_empty()` to check if any actual elements overlap.
    fn overlaps(&self, other: &Self) -> bool;
    fn is_adjacent(&self, other: &Self) -> bool;
    /// Returns true if `self` fully contains `other` as a sub-range.
    fn contains_range(&self, other: &Self) -> bool;

    /// The returned range will attempt to be a subset of `self`, potentially normalised,
    /// but this is not always possible.
    /// (For example `(0u8..=0).intersection(3..=3)` is empty but cannot be 'based' at `0`).
    ///
    /// TODO: is attempting to do that worth it? We give up `a.intersection(b) == b.intersection(a)`.
    fn intersection(self, other: Self) -> Self;
    /// `self.union(other) == other.union(self)`
    fn union(self, other: Self) -> Option<Self>;
}
impl<T> RangeExt for Range<T>
where
    T: Ord + Add<Output = T> + Sub<Output = T> + Clone,
{
    type Item = T;

    fn normalize(self) -> Self {
        if self.start > self.end {
            self.start.clone()..self.start
        } else {
            self
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => self.start == other.start,
            (true, false) => other.contains(&self.start),
            (false, true) => self.contains(&other.start),
            (false, false) => self.start < other.end && other.start < self.end,
        }
    }
    fn is_adjacent(&self, other: &Self) -> bool {
        let s = self.clone().normalize();
        let o = other.clone().normalize();
        s.end == o.start || o.end == s.start
    }
    fn contains_range(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    fn intersection(self, other: Self) -> Self {
        if self.is_empty() {
            self.start.clone()..self.start
        } else if other.is_empty() {
            if self.contains(&other.start) {
                other.start.clone()..other.start
            } else {
                self.start.clone()..self.start
            }
        } else {
            Ord::max(self.start, other.start)..Ord::min(self.end, other.end)
        }
    }
    fn union(self, other: Self) -> Option<Self> {
        Some(match (self.is_empty(), other.is_empty()) {
            // handle both empty to ensure a.union(b) == b.union(a).
            (true, true) => Ord::max(self.start, other.start)..Ord::min(self.end, other.end),
            (true, false) => other,
            (false, true) => self,
            (false, false) if self.overlaps(&other) || self.is_adjacent(&other) => {
                Ord::min(self.start, other.start)..Ord::max(self.end, other.end)
            }
            (false, false) => return None,
        })
    }
}
impl<T> RangeExt for RangeInclusive<T>
where
    T: Ord + Step + Clone + std::fmt::Debug,
{
    type Item = T;

    fn normalize(self) -> Self {
        let Some(start_back) = T::backward_checked(self.start().clone(), 1) else {
            return self;
        };
        if self.end() < &start_back {
            self.start().clone()..=start_back
        } else {
            self
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => self.start() == other.start(),
            (true, false) => other.contains(self.start()),
            (false, true) => self.contains(other.start()),
            (false, false) => self.start() <= other.end() && other.start() <= self.end(),
        }
    }

    fn is_adjacent(&self, other: &Self) -> bool {
        let s = self.clone().normalize();
        let o = other.clone().normalize();

        &T::forward(s.end().clone(), 1) == o.start() || &T::forward(o.end().clone(), 1) == s.start()
    }
    fn contains_range(&self, other: &Self) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    fn intersection(self, other: Self) -> Self {
        Ord::max(self.start().clone(), other.start().clone())
            ..=Ord::min(self.end().clone(), other.end().clone())
    }

    fn union(self, other: Self) -> Option<Self> {
        Some(match (self.is_empty(), other.is_empty()) {
            // handle both empty to ensure a.union(b) == b.union(a).
            (true, true) => {
                Ord::max(self.start().clone(), other.start().clone())
                    ..=Ord::min(self.end().clone(), other.end().clone())
            }
            (true, false) => other,
            (false, true) => self,
            (false, false) if self.overlaps(&other) || self.is_adjacent(&other) => {
                Ord::min(self.start().clone(), other.start().clone())
                    ..=Ord::max(self.end().clone(), other.end().clone())
            }
            (false, false) => return None,
        })
    }
}
pub trait RangeLen {
    type Output;
    // NOTE: named `range_len` instead of `len` to avoid name collisions.
    fn range_len(&self) -> Self::Output;
}
macro_rules! range_len {
    ($t:ty) => {
        impl RangeLen for Range<$t> {
            type Output = $t;
            fn range_len(&self) -> Self::Output {
                if self.is_empty() {
                    0
                } else {
                    self.end - self.start
                }
            }
        }
    };
}
range_len!(u8);
range_len!(u16);
range_len!(u32);
range_len!(u64);
range_len!(usize);
range_len!(i8);
range_len!(i16);
range_len!(i32);
range_len!(i64);
range_len!(isize);

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::{rngs::SmallRng, Rng, SeedableRng};

    use crate::range::RangeExt;

    #[test]
    fn test_range_random_intersection() {
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..10_000_000 {
            let a = rng.random_range(0..7)..rng.random_range(0..7);
            let b = rng.random_range(0..7)..rng.random_range(0..7);

            let intersection = a.clone().intersection(b.clone());
            let intersection_rev = b.clone().intersection(a.clone());

            let overlap: HashSet<_> = a.clone().filter(|a| b.contains(a)).collect();

            // println!("a: {a:?}, b: {b:?}, intersection: {intersection:?}, overlap: {overlap:?}");

            assert_eq!(a.overlaps(&b), b.overlaps(&a));
            assert_eq!(a.is_adjacent(&b), b.is_adjacent(&a));
            if !a.is_empty() && !b.is_empty() {
                // Empty ranges can both overlap and be adjacent.
                // For example `0..0` and `0..2` should be both overlapping and adjacent.
                // (Overlapping for the same reason `0..5` and `3..3` are overlapping.)
                if a.overlaps(&b) {
                    assert!(!a.is_adjacent(&b));
                    assert!(!b.is_adjacent(&a));
                }
                if a.is_adjacent(&b) {
                    assert!(!a.overlaps(&b));
                    assert!(!b.overlaps(&a));
                }
            }

            if !intersection.is_empty() {
                assert!(a.overlaps(&b));
                assert!(b.overlaps(&a));
            }

            if a.is_empty() || b.is_empty() {
                assert!(intersection.is_empty());
                assert!(intersection_rev.is_empty());
                assert!(overlap.is_empty());
            } else {
                assert_eq!(intersection, intersection_rev);
            }

            assert_eq!(
                intersection.clone().collect::<HashSet<_>>(),
                intersection_rev.clone().collect()
            );
            assert_eq!(overlap, intersection.clone().collect());

            if overlap.is_empty() {
                assert!(!a.overlaps(&b) || a.is_empty() || b.is_empty());
                assert!(intersection.is_empty());
            } else {
                assert!(a.overlaps(&b));
                assert!(!intersection.is_empty());
            }
        }
    }

    #[test]
    fn test_range_random_union() {
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..10_000_000 {
            let a = rng.random_range(0..7)..rng.random_range(0..7);
            let b = rng.random_range(0..7)..rng.random_range(0..7);

            let union = a.clone().union(b.clone());
            let union_rev = b.clone().union(a.clone());

            let overlap: HashSet<_> = a.clone().filter(|a| b.contains(a)).collect();
            let all: HashSet<_> = a.clone().chain(b.clone()).collect();

            // println!("a: {a:?}, b: {b:?}, overlap: {overlap:?}, union: {union:?}, all: {all:?}");

            if overlap.is_empty() {
                if let Some(union) = union.clone() {
                    if !(union.is_empty() || a.is_empty() || b.is_empty()) {
                        assert!(a.is_adjacent(&b));
                        assert!(b.is_adjacent(&a));
                    }
                } else {
                    assert!(!a.is_adjacent(&b));
                    assert!(!b.is_adjacent(&a));
                }
            }

            assert_eq!(union, union_rev);
            if let Some(union) = union.clone() {
                assert_eq!(all, union.collect());
            } else {
                let min = all.iter().min().copied().unwrap();
                let max = all.iter().max().copied().unwrap();
                assert!(all.len() < (max + 1) - min);
                assert!(overlap.is_empty());
            }
        }
    }

    #[test]
    fn test_range_inclusive_random_intersection() {
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..10_000_000 {
            let a = rng.random_range(0..7)..=rng.random_range(0..7);
            let b = rng.random_range(0..7)..=rng.random_range(0..7);

            let intersection = a.clone().intersection(b.clone());
            let intersection_rev = b.clone().intersection(a.clone());

            let overlap: HashSet<_> = a.clone().filter(|a| b.contains(a)).collect();

            // println!("a: {a:?}, b: {b:?}, intersection: {intersection:?}, intersection_rev: {intersection_rev:?}, overlap: {overlap:?}");

            assert_eq!(intersection, intersection_rev); // Note: not asserted for non-inclusive ranges.

            assert_eq!(a.overlaps(&b), b.overlaps(&a));
            assert_eq!(a.is_adjacent(&b), b.is_adjacent(&a));
            if !a.is_empty() && !b.is_empty() {
                // Empty ranges can both overlap and be adjacent.
                // For example `0..0` and `0..2` should be both overlapping and adjacent.
                // (Overlapping for the same reason `0..5` and `3..3` are overlapping.)
                if a.overlaps(&b) {
                    assert!(!a.is_adjacent(&b));
                    assert!(!b.is_adjacent(&a));
                }
                if a.is_adjacent(&b) {
                    assert!(!a.overlaps(&b));
                    assert!(!b.overlaps(&a));
                }
            }

            if !intersection.is_empty() {
                assert!(a.overlaps(&b));
                assert!(b.overlaps(&a));
            }

            if a.is_empty() || b.is_empty() {
                assert!(intersection.is_empty());
                assert!(intersection_rev.is_empty());
                assert!(overlap.is_empty());
            } else {
                assert_eq!(intersection, intersection_rev);
            }

            assert_eq!(
                intersection.clone().collect::<HashSet<_>>(),
                intersection_rev.clone().collect()
            );
            assert_eq!(overlap, intersection.clone().collect());

            if overlap.is_empty() {
                assert!(!a.overlaps(&b) || a.is_empty() || b.is_empty());
                assert!(intersection.is_empty());
            } else {
                assert!(a.overlaps(&b));
                assert!(!intersection.is_empty());
            }
        }
    }

    #[test]
    fn test_range_inclusive_random_union() {
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..10_000_000 {
            let a = rng.random_range(0..7)..=rng.random_range(0..7);
            let b = rng.random_range(0..7)..=rng.random_range(0..7);

            let union = a.clone().union(b.clone());
            let union_rev = b.clone().union(a.clone());

            let overlap: HashSet<_> = a.clone().filter(|a| b.contains(a)).collect();
            let all: HashSet<_> = a.clone().chain(b.clone()).collect();

            // println!("a: {a:?}, b: {b:?}, overlap: {overlap:?}, union: {union:?}, all: {all:?}");

            if overlap.is_empty() {
                if let Some(union) = union.clone() {
                    if !(union.is_empty() || a.is_empty() || b.is_empty()) {
                        assert!(a.is_adjacent(&b));
                        assert!(b.is_adjacent(&a));
                    }
                } else {
                    assert!(!a.is_adjacent(&b));
                    assert!(!b.is_adjacent(&a));
                }
            }

            assert_eq!(union, union_rev);
            if let Some(union) = union.clone() {
                assert_eq!(all, union.collect());
            } else {
                let min = all.iter().min().copied().unwrap();
                let max = all.iter().max().copied().unwrap();
                assert!(all.len() < (max + 1) - min);
                assert!(overlap.is_empty());
            }
        }
    }
}
