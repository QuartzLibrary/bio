use std::{fmt, ops::Range};

use serde::{Deserialize, Serialize};
use utile::range::RangeExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct ContigPosition<Contig = String> {
    pub contig: Contig,
    pub at: u64,
}
impl<Contig> ContigPosition<Contig> {
    pub fn map_contig<NewContig>(
        self,
        f: impl FnOnce(Contig) -> NewContig,
    ) -> ContigPosition<NewContig> {
        ContigPosition {
            contig: f(self.contig),
            at: self.at,
        }
    }

    pub fn as_ref_contig(&self) -> ContigPosition<&Contig> {
        ContigPosition {
            contig: &self.contig,
            at: self.at,
        }
    }

    pub fn into_range_to(self, other: &Self) -> ContigRange<Contig>
    where
        Contig: PartialEq + fmt::Debug,
    {
        assert_eq!(self.contig, other.contig);
        ContigRange {
            contig: self.contig,
            at: self.at..other.at,
        }
    }

    pub fn usize_pos(&self) -> usize {
        usize::try_from(self.at).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContigRange<Contig = String> {
    pub contig: Contig,
    pub at: Range<u64>,
}
impl<Contig> ContigRange<Contig> {
    pub fn len(&self) -> u64 {
        let &Range { start, end } = &self.at;
        end.saturating_sub(start)
    }
    pub fn is_empty(&self) -> bool {
        self.at.is_empty()
    }

    pub fn into_start(self) -> ContigPosition<Contig> {
        ContigPosition {
            contig: self.contig,
            at: self.at.start,
        }
    }
    pub fn into_end(self) -> ContigPosition<Contig> {
        ContigPosition {
            contig: self.contig,
            at: self.at.end,
        }
    }

    pub fn usize_range(&self) -> Range<usize> {
        usize::try_from(self.at.start).unwrap()..usize::try_from(self.at.end).unwrap()
    }

    pub fn contains(&self, loc: &ContigPosition<Contig>) -> bool
    where
        Contig: PartialEq,
    {
        self.contig == loc.contig && self.at.contains(&loc.at)
    }
    pub fn contains_range(&self, range: &Self) -> bool
    where
        Contig: PartialEq,
    {
        self.contig == range.contig && self.at.contains_range(&range.at)
    }

    pub fn intersection(self, b: &Self) -> Option<Self>
    where
        Contig: PartialEq,
    {
        if self.contig != b.contig {
            return None;
        }

        Some(Self {
            contig: self.contig,
            at: self.at.intersection(b.at.clone()),
        })
    }
    pub fn overlaps(&self, b: &Self) -> bool
    where
        Contig: PartialEq,
    {
        self.contig == b.contig && self.at.overlaps(&b.at)
    }

    pub fn map_contig<NewContig>(
        self,
        f: impl FnOnce(Contig) -> NewContig,
    ) -> ContigRange<NewContig> {
        ContigRange {
            contig: f(self.contig),
            at: self.at,
        }
    }
    pub fn map_range(self, f: impl FnOnce(Range<u64>) -> Range<u64>) -> Self {
        Self {
            contig: self.contig,
            at: f(self.at),
        }
    }

    pub fn as_ref_contig(&self) -> ContigRange<&Contig> {
        ContigRange {
            contig: &self.contig,
            at: self.at.clone(),
        }
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = ContigPosition<Contig>>
    where
        Contig: Clone,
    {
        let ContigRange {
            contig,
            at: Range { start, end },
        } = self.clone();
        (start..end).map(move |at| ContigPosition {
            contig: contig.clone(),
            at,
        })
    }
}
impl<Contig> PartialOrd for ContigRange<Contig>
where
    Contig: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Self { contig: _, at: _ } = self; // Exhaustiveness check

        Some(
            PartialOrd::partial_cmp(&self.contig, &other.contig)?
                .then_with(|| Ord::cmp(&self.at.start, &other.at.start))
                .then_with(|| Ord::cmp(&self.at.end, &other.at.end)),
        )
    }
}
impl<Contig> Ord for ContigRange<Contig>
where
    Contig: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Self { contig: _, at: _ } = self; // Exhaustiveness check

        Ord::cmp(&self.contig, &other.contig)
            .then_with(|| Ord::cmp(&self.at.start, &other.at.start))
            .then_with(|| Ord::cmp(&self.at.end, &other.at.end))
    }
}

impl<Contig> From<ContigPosition<Contig>> for ContigRange<Contig> {
    fn from(loc: ContigPosition<Contig>) -> Self {
        Self {
            contig: loc.contig,
            at: loc.at..(loc.at + 1),
        }
    }
}
#[derive(Debug, Clone, thiserror::Error)]
#[error("Expected a single base location, but found a range {from:?}.")]
pub struct LocationConversionError<Contig> {
    pub from: ContigRange<Contig>,
}
impl<Contig> TryFrom<ContigRange<Contig>> for ContigPosition<Contig> {
    type Error = LocationConversionError<Contig>;
    fn try_from(range: ContigRange<Contig>) -> Result<Self, Self::Error> {
        if range.at.start + 1 != range.at.end {
            return Err(LocationConversionError { from: range });
        }
        Ok(Self {
            contig: range.contig,
            at: range.at.start,
        })
    }
}

pub mod orientation {
    use serde::{Deserialize, Serialize};

    use crate::genome::Contig;

    use super::{ContigPosition, ContigRange};

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

    impl<C> WithOrientation<ContigPosition<C>>
    where
        C: Contig,
    {
        #[track_caller]
        pub fn set_orientation(&mut self, orientation: SequenceOrientation) {
            self.set_orientation_with(orientation, self.v.contig.size());
        }
        #[track_caller]
        pub fn flip_orientation(self) -> Self {
            let size = self.v.contig.size();
            self.flip_orientation_with(size)
        }
    }
    impl<C> WithOrientation<ContigPosition<C>> {
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
                v: ContigPosition {
                    contig: self.v.contig,
                    at: size - self.v.at - 1,
                },
            }
        }

        pub fn as_ref_contig(&self) -> WithOrientation<ContigPosition<&C>> {
            WithOrientation {
                orientation: self.orientation,
                v: self.v.as_ref_contig(),
            }
        }
    }

    impl<C> WithOrientation<ContigRange<C>>
    where
        C: Contig,
    {
        pub fn set_orientation(&mut self, orientation: SequenceOrientation) {
            self.set_orientation_with(orientation, self.v.contig.size());
        }
        pub fn flip_orientation(self) -> Self {
            let size = self.v.contig.size();
            self.flip_orientation_with(size)
        }

        pub fn contains(&self, loc: &WithOrientation<ContigPosition<C>>) -> bool
        where
            C: PartialEq,
        {
            self.contains_with(loc, self.v.contig.size())
        }
        pub fn contains_range(&self, range: &Self) -> bool
        where
            C: PartialEq,
        {
            self.contains_range_with(range, self.v.contig.size())
        }

        pub fn intersection(self, b: &Self) -> Option<Self>
        where
            C: PartialEq,
        {
            let size = self.v.contig.size();
            self.intersection_with(b, size)
        }
        pub fn overlaps(&self, b: &Self) -> bool
        where
            C: PartialEq,
        {
            self.overlaps_with(b, self.v.contig.size())
        }

        pub fn iter_positions(&self) -> impl Iterator<Item = WithOrientation<ContigPosition<C>>>
        where
            C: Clone,
        {
            let orientation = self.orientation;
            self.v.iter_positions().map(move |pos| WithOrientation {
                orientation,
                v: pos,
            })
        }
    }
    impl<C> WithOrientation<ContigRange<C>> {
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
                v: ContigRange {
                    contig: self.v.contig,
                    at: (size - self.v.at.end)..(size - self.v.at.start),
                },
            }
        }

        pub fn contains_with(&self, pos: &WithOrientation<ContigPosition<C>>, size: u64) -> bool
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
            if self.v.contig != b.v.contig {
                return None;
            }

            let mut b = b.as_ref_contig();

            b.set_orientation_with(self.orientation, size);

            let ContigRange { contig: _, at } = self.as_ref_contig().v.intersection(&b.v)?;

            Some(Self {
                orientation: self.orientation,
                v: ContigRange {
                    contig: self.v.contig,
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

        pub fn as_ref_contig(&self) -> WithOrientation<ContigRange<&C>> {
            WithOrientation {
                orientation: self.orientation,
                v: self.v.as_ref_contig(),
            }
        }
    }
}

mod math {
    use std::ops::{Add, AddAssign, Sub, SubAssign};

    use crate::genome::Contig;

    use super::*;

    impl<C: Contig> ContigPosition<C> {
        pub fn checked_add(self, rhs: u64) -> Option<Self> {
            if self.contig.size() <= self.at + rhs {
                None
            } else {
                Some(Self {
                    contig: self.contig,
                    at: self.at + rhs,
                })
            }
        }
    }
    impl<C: Contig> Add<u64> for ContigPosition<C> {
        type Output = Self;

        fn add(self, rhs: u64) -> Self::Output {
            self.checked_add(rhs).unwrap()
        }
    }
    impl<C: Contig> AddAssign<u64> for ContigPosition<C> {
        fn add_assign(&mut self, rhs: u64) {
            assert!(self.at + rhs <= self.contig.size());
            self.at += rhs;
        }
    }

    impl<C> ContigPosition<C> {
        pub fn checked_sub(self, rhs: u64) -> Option<Self> {
            Some(Self {
                contig: self.contig,
                at: self.at.checked_sub(rhs)?,
            })
        }
    }
    impl<C> Sub<u64> for ContigPosition<C> {
        type Output = Self;

        fn sub(self, rhs: u64) -> Self::Output {
            self.checked_sub(rhs).unwrap()
        }
    }
    impl<C> SubAssign<u64> for ContigPosition<C> {
        fn sub_assign(&mut self, rhs: u64) {
            assert!(self.at >= rhs);
            self.at -= rhs;
        }
    }

    impl<C: Contig> ContigRange<C> {
        pub fn checked_add(self, rhs: u64) -> Option<Self> {
            if self.contig.size() <= self.at.end + rhs {
                None
            } else {
                Some(Self {
                    contig: self.contig,
                    at: self.at.start + rhs..self.at.end + rhs,
                })
            }
        }
    }
    impl<C: Contig> Add<u64> for ContigRange<C> {
        type Output = Self;

        fn add(self, rhs: u64) -> Self::Output {
            self.checked_add(rhs).unwrap()
        }
    }
    impl<C: Contig> AddAssign<u64> for ContigRange<C> {
        fn add_assign(&mut self, rhs: u64) {
            assert!(self.at.end + rhs <= self.contig.size());
            self.at.start += rhs;
            self.at.end += rhs;
        }
    }

    impl<C> ContigRange<C> {
        pub fn checked_sub(self, rhs: u64) -> Option<Self> {
            Some(Self {
                contig: self.contig,
                at: self.at.start.checked_sub(rhs)?..self.at.end.checked_sub(rhs)?,
            })
        }
    }
    impl<C> Sub<u64> for ContigRange<C> {
        type Output = Self;

        fn sub(self, rhs: u64) -> Self::Output {
            self.checked_sub(rhs).unwrap()
        }
    }
    impl<C> SubAssign<u64> for ContigRange<C> {
        fn sub_assign(&mut self, rhs: u64) {
            assert!(self.at.start >= rhs);
            self.at.start -= rhs;
            self.at.end -= rhs;
        }
    }
}

mod noodles {
    use std::{fmt, ops::Range};

    use noodles::core::{region::Interval, Position, Region};

    use super::{ContigPosition, ContigRange};

    impl<T: fmt::Display> fmt::Display for ContigPosition<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self { contig, at } = self;

            write!(f, "{contig}@{at}")
        }
    }
    impl<T: fmt::Display> fmt::Display for ContigRange<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                contig,
                at: Range { start, end },
            } = self;

            write!(f, "{contig}@{start}..{end}")
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

    impl<C> TryFrom<ContigRange<C>> for Region
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(ContigRange { contig, at }: ContigRange<C>) -> Result<Self, Self::Error> {
            if at.is_empty() {
                return Err(GenomeLocationConversionError::EmptyRange);
            }
            if at.end < at.start {
                return Err(GenomeLocationConversionError::InvalidRange);
            }

            let at = usize::try_from(at.start).unwrap()..usize::try_from(at.end).unwrap();

            let start = Position::new(at.start + 1).unwrap();
            let end = Position::new((at.end + 1) - 1).unwrap();
            Ok(Region::new(contig.as_ref(), Interval::from(start..=end)))
        }
    }
    impl<C> TryFrom<ContigPosition<C>> for Region
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(value: ContigPosition<C>) -> Result<Self, Self::Error> {
            let pos = usize::try_from(value.at).unwrap();
            let pos = Position::new(pos + 1).unwrap();
            Ok(Region::new(
                value.contig.as_ref(),
                Interval::from(pos..=pos),
            ))
        }
    }

    impl<C> TryFrom<ContigRange<C>> for Interval
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(value: ContigRange<C>) -> Result<Self, Self::Error> {
            (&value).try_into()
        }
    }
    impl<C> TryFrom<&ContigRange<C>> for Interval
    where
        C: AsRef<str>,
    {
        type Error = GenomeLocationConversionError;

        fn try_from(ContigRange { contig: _, at }: &ContigRange<C>) -> Result<Self, Self::Error> {
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
