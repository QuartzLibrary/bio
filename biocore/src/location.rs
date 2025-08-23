use std::ops::Range;

use serde::{Deserialize, Serialize};
use utile::range::RangeExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct GenomePosition<Contig = String> {
    pub name: Contig,
    pub at: u64,
}
impl<Contig> GenomePosition<Contig> {
    pub fn map_contig<NewContig>(
        self,
        f: impl FnOnce(Contig) -> NewContig,
    ) -> GenomePosition<NewContig> {
        GenomePosition {
            name: f(self.name),
            at: self.at,
        }
    }

    pub fn as_ref_contig(&self) -> GenomePosition<&Contig> {
        GenomePosition {
            name: &self.name,
            at: self.at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenomeRange<Contig = String> {
    pub name: Contig,
    pub at: Range<u64>,
}
impl<Contig> GenomeRange<Contig> {
    pub fn len(&self) -> u64 {
        let &Range { start, end } = &self.at;
        end.saturating_sub(start)
    }
    pub fn is_empty(&self) -> bool {
        self.at.is_empty()
    }
    pub fn contains(&self, loc: &GenomePosition<Contig>) -> bool
    where
        Contig: PartialEq,
    {
        self.name == loc.name && self.at.contains(&loc.at)
    }
    pub fn contains_range(&self, range: &Self) -> bool
    where
        Contig: PartialEq,
    {
        self.name == range.name
            && self.at.contains(&range.at.start)
            && (self.at.contains(&range.at.end) || self.at.end == range.at.end)
    }

    pub fn intersection(self, b: &Self) -> Option<Self>
    where
        Contig: PartialEq,
    {
        if self.name != b.name {
            return None;
        }

        Some(Self {
            name: self.name,
            at: self.at.intersection(b.at.clone()),
        })
    }
    pub fn overlaps(&self, b: &Self) -> bool
    where
        Contig: PartialEq,
    {
        self.name == b.name && self.at.overlaps(&b.at)
    }

    pub fn map_contig<NewContig>(
        self,
        f: impl FnOnce(Contig) -> NewContig,
    ) -> GenomeRange<NewContig> {
        GenomeRange {
            name: f(self.name),
            at: self.at,
        }
    }

    pub fn as_ref_contig(&self) -> GenomeRange<&Contig> {
        GenomeRange {
            name: &self.name,
            at: self.at.clone(),
        }
    }
}
impl<Contig> PartialOrd for GenomeRange<Contig>
where
    Contig: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Self { name: _, at: _ } = self; // Exhaustiveness check

        Some(
            PartialOrd::partial_cmp(&self.name, &other.name)?
                .then_with(|| Ord::cmp(&self.at.start, &other.at.start))
                .then_with(|| Ord::cmp(&self.at.end, &other.at.end)),
        )
    }
}
impl<Contig> Ord for GenomeRange<Contig>
where
    Contig: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Self { name: _, at: _ } = self; // Exhaustiveness check

        Ord::cmp(&self.name, &other.name)
            .then_with(|| Ord::cmp(&self.at.start, &other.at.start))
            .then_with(|| Ord::cmp(&self.at.end, &other.at.end))
    }
}

impl<Contig> From<GenomePosition<Contig>> for GenomeRange<Contig> {
    fn from(loc: GenomePosition<Contig>) -> Self {
        Self {
            name: loc.name,
            at: loc.at..(loc.at + 1),
        }
    }
}
#[derive(Debug, Clone, thiserror::Error)]
#[error("Expected a single base location, but found a range {from:?}.")]
pub struct LocationConversionError<Contig> {
    pub from: GenomeRange<Contig>,
}
impl<Contig> TryFrom<GenomeRange<Contig>> for GenomePosition<Contig> {
    type Error = LocationConversionError<Contig>;
    fn try_from(range: GenomeRange<Contig>) -> Result<Self, Self::Error> {
        if range.at.start + 1 != range.at.end {
            return Err(LocationConversionError { from: range });
        }
        Ok(Self {
            name: range.name,
            at: range.at.start,
        })
    }
}

pub mod orientation {
    use std::ops::{Deref, DerefMut};

    use serde::{Deserialize, Serialize};

    use crate::genome::Contig;

    use super::{GenomePosition, GenomeRange};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[derive(Serialize, Deserialize)]
    pub struct WithOrientation<T> {
        pub orientation: SequenceOrientation,
        pub v: T,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[derive(Serialize, Deserialize)]
    pub enum SequenceOrientation {
        /// 5' -> 3'
        Forward,
        /// 3' -> 5'
        Reverse,
    }

    impl SequenceOrientation {
        pub fn is_forward(self) -> bool {
            self == Self::Forward
        }
        pub fn is_reverse(self) -> bool {
            self == Self::Reverse
        }
        pub fn flip(self) -> Self {
            match self {
                Self::Forward => Self::Reverse,
                Self::Reverse => Self::Forward,
            }
        }
    }

    impl<T> WithOrientation<T> {
        pub fn new_forward(v: T) -> Self {
            Self {
                orientation: SequenceOrientation::Forward,
                v,
            }
        }
        pub fn new_reverse(v: T) -> Self {
            Self {
                orientation: SequenceOrientation::Reverse,
                v,
            }
        }

        pub fn is_forward(&self) -> bool {
            self.orientation == SequenceOrientation::Forward
        }
        pub fn is_reverse(&self) -> bool {
            self.orientation == SequenceOrientation::Reverse
        }

        pub fn map_value<O>(self, f: impl FnOnce(T) -> O) -> WithOrientation<O> {
            WithOrientation {
                orientation: self.orientation,
                v: f(self.v),
            }
        }
    }

    impl<C> WithOrientation<GenomePosition<C>>
    where
        C: Contig,
    {
        #[track_caller]
        pub fn set_orientation(&mut self, orientation: SequenceOrientation) {
            self.set_orientation_with(orientation, self.v.name.size());
        }
        #[track_caller]
        pub fn flip_orientation(self) -> Self {
            let size = self.v.name.size();
            self.flip_orientation_with(size)
        }
    }
    impl<C> WithOrientation<GenomePosition<C>> {
        #[track_caller]
        pub fn set_orientation_with(&mut self, orientation: SequenceOrientation, size: u64) {
            if self.orientation != orientation {
                self.orientation = self.orientation.flip();
                self.v.at = size - self.v.at - 1;
            }
        }
        #[track_caller]
        pub fn flip_orientation_with(self, size: u64) -> Self {
            Self {
                orientation: self.orientation.flip(),
                v: GenomePosition {
                    name: self.v.name,
                    at: size - self.v.at - 1,
                },
            }
        }

        pub fn as_ref_contig(&self) -> WithOrientation<GenomePosition<&C>> {
            WithOrientation {
                orientation: self.orientation,
                v: self.v.as_ref_contig(),
            }
        }
    }

    impl<C> WithOrientation<GenomeRange<C>>
    where
        C: Contig,
    {
        pub fn set_orientation(&mut self, orientation: SequenceOrientation) {
            self.set_orientation_with(orientation, self.v.name.size());
        }
        pub fn flip_orientation(self) -> Self {
            let size = self.v.name.size();
            self.flip_orientation_with(size)
        }

        pub fn contains(&self, loc: &WithOrientation<GenomePosition<C>>) -> bool
        where
            C: PartialEq,
        {
            self.contains_with(loc, self.v.name.size())
        }
        pub fn contains_range(&self, range: &Self) -> bool
        where
            C: PartialEq,
        {
            self.contains_range_with(range, self.v.name.size())
        }

        pub fn intersection(self, b: &Self) -> Option<Self>
        where
            C: PartialEq,
        {
            let size = self.v.name.size();
            self.intersection_with(b, size)
        }
        pub fn overlaps(&self, b: &Self) -> bool
        where
            C: PartialEq,
        {
            self.overlaps_with(b, self.v.name.size())
        }
    }
    impl<C> WithOrientation<GenomeRange<C>> {
        pub fn set_orientation_with(&mut self, orientation: SequenceOrientation, size: u64) {
            if self.orientation != orientation {
                if size == 0 {
                    assert_eq!(0, self.v.at.start);
                    assert_eq!(0, self.v.at.end);
                }

                self.orientation = self.orientation.flip();
                self.v.at = (size - self.v.at.end)..(size - self.v.at.start);
            }
        }
        pub fn flip_orientation_with(self, size: u64) -> Self {
            if size == 0 {
                assert_eq!(0, self.v.at.start);
                assert_eq!(0, self.v.at.end);
            }
            // 0 1 2 3 4 5 6 7 8 9
            // 9 8 7 6 5 4 3 2 1 0
            // 0..1 -> 9..10
            // 9..10 -> 0..1

            Self {
                orientation: self.orientation.flip(),
                v: GenomeRange {
                    name: self.v.name,
                    at: (size - self.v.at.end)..(size - self.v.at.start),
                },
            }
        }

        pub fn contains_with(&self, pos: &WithOrientation<GenomePosition<C>>, size: u64) -> bool
        where
            C: PartialEq,
        {
            let mut pos = pos.as_ref_contig();
            if self.orientation != pos.orientation {
                pos = pos.flip_orientation_with(size)
            };
            self.v.as_ref_contig().contains(&pos.v)
        }
        pub fn contains_range_with(&self, range: &Self, size: u64) -> bool
        where
            C: PartialEq,
        {
            let mut range = range.as_ref_contig();
            if self.orientation != range.orientation {
                range = range.clone().flip_orientation_with(size)
            };
            self.v.as_ref_contig().contains_range(&range.v)
        }

        /// Preserves the orientation of `self`.
        pub fn intersection_with(self, b: &Self, size: u64) -> Option<Self>
        where
            C: PartialEq,
        {
            if self.v.name != b.v.name {
                return None;
            }

            let mut b = b.as_ref_contig();

            b.set_orientation_with(self.orientation, size);

            let GenomeRange { name: _, at } = self.as_ref_contig().v.intersection(&b.v)?;

            Some(Self {
                orientation: self.orientation,
                v: GenomeRange {
                    name: self.v.name,
                    at,
                },
            })
        }
        pub fn overlaps_with(&self, b: &Self, size: u64) -> bool
        where
            C: PartialEq,
        {
            let mut b = b.as_ref_contig();
            b.set_orientation_with(self.orientation, size);
            self.v.as_ref_contig().overlaps(&b.v)
        }

        pub fn as_ref_contig(&self) -> WithOrientation<GenomeRange<&C>> {
            WithOrientation {
                orientation: self.orientation,
                v: self.v.as_ref_contig(),
            }
        }
    }

    impl<T> Deref for WithOrientation<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.v
        }
    }
    impl<T> DerefMut for WithOrientation<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.v
        }
    }
}

mod noodles {
    use std::{fmt, ops::Range};

    use noodles::core::{region::Interval, Position, Region};

    use super::{GenomePosition, GenomeRange};

    impl<T: fmt::Display> fmt::Display for GenomePosition<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self { name, at } = self;

            write!(f, "{name}@{at}")
        }
    }
    impl<T: fmt::Display> fmt::Display for GenomeRange<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                name,
                at: Range { start, end },
            } = self;

            write!(f, "{name}@{start}..{end}")
        }
    }

    #[derive(Debug, Clone, thiserror::Error)]
    pub enum GenomeLocationConversionError {
        #[error("Cannot convert reverse-oriented range to noodles Region.")]
        ReverseOrientation,
        #[error("Cannot convert empty range to noodles Region.")]
        EmptyRange,
        #[error("Cannot convert invalid range (end < start) to noodles Region.")]
        InvalidRange,
    }

    impl<C> TryFrom<GenomeRange<C>> for Region
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(GenomeRange { name, at }: GenomeRange<C>) -> Result<Self, Self::Error> {
            if at.is_empty() {
                return Err(GenomeLocationConversionError::EmptyRange);
            }
            if at.end < at.start {
                return Err(GenomeLocationConversionError::InvalidRange);
            }

            let at = usize::try_from(at.start).unwrap()..usize::try_from(at.end).unwrap();

            let start = Position::new(at.start + 1).unwrap();
            let end = Position::new((at.end + 1) - 1).unwrap();
            Ok(Region::new(name.as_ref(), Interval::from(start..=end)))
        }
    }
    impl<C> TryFrom<GenomePosition<C>> for Region
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(value: GenomePosition<C>) -> Result<Self, Self::Error> {
            let pos = usize::try_from(value.at).unwrap();
            let pos = Position::new(pos + 1).unwrap();
            Ok(Region::new(value.name.as_ref(), Interval::from(pos..=pos)))
        }
    }

    impl<C> TryFrom<GenomeRange<C>> for Interval
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(value: GenomeRange<C>) -> Result<Self, Self::Error> {
            (&value).try_into()
        }
    }
    impl<C> TryFrom<&GenomeRange<C>> for Interval
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(GenomeRange { name: _, at }: &GenomeRange<C>) -> Result<Self, Self::Error> {
            if at.is_empty() {
                return Err(GenomeLocationConversionError::EmptyRange);
            }
            if at.end < at.start {
                return Err(GenomeLocationConversionError::InvalidRange);
            }

            let at = usize::try_from(at.start).unwrap()..usize::try_from(at.end).unwrap();

            let start = Position::new(at.start + 1).unwrap();
            let end = Position::new((at.end + 1) - 1).unwrap();
            Ok(Interval::from(start..=end))
        }
    }

    // impl From<Region> for GenomeRange {
    //     fn from(value: Region) -> Self {
    //         let start = value
    //             .interval()
    //             .start()
    //             .map(|p| u64::try_from(p.get() - 1).unwrap())
    //             .unwrap_or(0);
    //         let end = value
    //             .interval()
    //             .end()
    //             .map(|p| u64::try_from((p.get() - 1) + 1).unwrap())
    //             .unwrap_or(u64::MAX);
    //         GenomeRange {
    //             name: String::from_utf8(value.name().to_vec()).unwrap(),
    //             at: start..end,
    //         }
    //     }
    // }
    // impl TryFrom<Region> for GenomePosition {
    //     type Error = super::LocationConversionError;

    //     fn try_from(value: Region) -> Result<Self, Self::Error> {
    //         let range: GenomeRange = value.into();
    //         range.try_into()
    //     }
    // }
}
