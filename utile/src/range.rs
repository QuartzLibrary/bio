use std::ops::Range;

pub trait RangeExt {
    fn overlaps(&self, other: &Self) -> bool;
    fn intersect(self, other: Self) -> Self;
}
impl<T: Ord> RangeExt for Range<T> {
    fn overlaps(&self, other: &Self) -> bool {
        self.contains(&other.start) || other.contains(&self.start)
    }

    fn intersect(self, other: Self) -> Self {
        Ord::max(self.start, other.start)..Ord::min(self.end, other.end)
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
