use std::ops::Range;

use serde::{Deserialize, Serialize};
use utile::range::RangeExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct GenomePosition<Contig = String> {
    pub name: Contig,
    pub orientation: SequenceOrientation,
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
            orientation: self.orientation,
        }
    }

    #[track_caller]
    pub fn set_orientation(&mut self, orientation: SequenceOrientation, size: u64) {
        if self.orientation != orientation {
            self.orientation = self.orientation.flip();
            self.at = size - self.at - 1;
        }
    }
    #[track_caller]
    pub fn flip_orientation(self, size: u64) -> Self {
        Self {
            name: self.name,
            at: size - self.at - 1,
            orientation: self.orientation.flip(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenomeRange<Contig = String> {
    pub name: Contig,
    pub orientation: SequenceOrientation,
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
    pub fn contains(&self, loc: &GenomePosition<Contig>, size: u64) -> bool
    where
        Contig: PartialEq + Clone,
    {
        self.name == loc.name && {
            let loc = if self.orientation == loc.orientation {
                loc
            } else {
                &loc.clone().flip_orientation(size)
            };
            self.at.contains(&loc.at)
        }
    }
    pub fn contains_range(&self, range: &Self, size: u64) -> bool
    where
        Contig: PartialEq + Clone,
    {
        self.name == range.name && {
            let range = if self.orientation == range.orientation {
                range
            } else {
                &range.clone().flip_orientation(size)
            };
            self.at.contains(&range.at.start)
                && (self.at.contains(&range.at.end) || self.at.end == range.at.end)
        }
    }

    pub fn map_contig<NewContig>(
        self,
        f: impl FnOnce(Contig) -> NewContig,
    ) -> GenomeRange<NewContig> {
        GenomeRange {
            name: f(self.name),
            at: self.at,
            orientation: self.orientation,
        }
    }

    #[track_caller]
    pub fn set_orientation(&mut self, orientation: SequenceOrientation, size: u64) {
        if self.orientation != orientation {
            self.orientation = self.orientation.flip();
            self.at = (size - self.at.end)..(size - self.at.start);
        }
    }
    #[track_caller]
    pub fn flip_orientation(self, size: u64) -> Self {
        if size == 0 {
            assert_eq!(0, self.at.start);
            assert_eq!(0, self.at.end);
        }
        // 0 1 2 3 4 5 6 7 8 9
        // 9 8 7 6 5 4 3 2 1 0
        // 0..1 -> 9..10
        // 9..10 -> 0..1
        Self {
            name: self.name,
            at: (size - self.at.end)..(size - self.at.start),
            orientation: self.orientation.flip(),
        }
    }
    /// Preserves the orientation of `self`.
    pub fn intersect(&self, b: Self, size: u64) -> Option<Self>
    where
        Contig: PartialEq,
    {
        if self.name != b.name {
            return None;
        }

        let b = if self.orientation != b.orientation {
            b.flip_orientation(size)
        } else {
            b
        };

        Some(Self {
            name: b.name,
            at: self.at.clone().intersection(b.at),
            orientation: self.orientation,
        })
    }
}
impl<Contig> PartialOrd for GenomeRange<Contig>
where
    Contig: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Self {
            name: _,
            at: _,
            orientation: _,
        } = self; // Exhaustiveness check

        Some(
            PartialOrd::partial_cmp(&self.name, &other.name)?
                .then_with(|| Ord::cmp(&self.orientation, &other.orientation))
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
        let Self {
            name: _,
            at: _,
            orientation: _,
        } = self; // Exhaustiveness check

        Ord::cmp(&self.name, &other.name)
            .then_with(|| Ord::cmp(&self.orientation, &other.orientation))
            .then_with(|| Ord::cmp(&self.at.start, &other.at.start))
            .then_with(|| Ord::cmp(&self.at.end, &other.at.end))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum SequenceOrientation {
    Forward,
    Reverse,
}
impl SequenceOrientation {
    pub fn is_forward(self) -> bool {
        self == Self::Forward
    }
    pub fn flip(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

impl From<GenomePosition> for GenomeRange {
    fn from(loc: GenomePosition) -> Self {
        Self {
            name: loc.name,
            at: loc.at..(loc.at + 1),
            orientation: loc.orientation,
        }
    }
}
#[derive(Debug, Clone, thiserror::Error)]
#[error("Expected a single base location, but found a range {from:?}.")]
pub struct LocationConversionError {
    pub from: GenomeRange,
}
impl TryFrom<GenomeRange> for GenomePosition {
    type Error = LocationConversionError;
    fn try_from(range: GenomeRange) -> Result<Self, Self::Error> {
        if range.at.start + 1 != range.at.end {
            return Err(LocationConversionError { from: range });
        }
        Ok(Self {
            name: range.name,
            at: range.at.start,
            orientation: range.orientation,
        })
    }
}

mod noodles {
    use std::{fmt, ops::Range};

    use noodles::core::{region::Interval, Position, Region};

    use super::{GenomePosition, GenomeRange};

    impl<T: fmt::Display> fmt::Display for GenomePosition<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                name,
                orientation,
                at,
            } = self;
            let o = if orientation.is_forward() {
                ""
            } else {
                "(reverse)"
            };
            write!(f, "{name}:{at}{o}")
        }
    }
    impl<T: fmt::Display> fmt::Display for GenomeRange<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                name,
                orientation,
                at: Range { start, end },
            } = self;
            let o = if orientation.is_forward() {
                ""
            } else {
                "(reverse)"
            };
            write!(f, "{name}:{start}..{end}{o}")
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

        fn try_from(
            GenomeRange {
                name,
                orientation,
                at,
            }: GenomeRange<C>,
        ) -> Result<Self, Self::Error> {
            if !orientation.is_forward() {
                return Err(GenomeLocationConversionError::ReverseOrientation);
            }
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
            if !value.orientation.is_forward() {
                return Err(GenomeLocationConversionError::ReverseOrientation);
            }

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

        fn try_from(
            GenomeRange {
                name: _,
                orientation: _,
                at,
            }: &GenomeRange<C>,
        ) -> Result<Self, Self::Error> {
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
    //             orientation: SequenceOrientation::Forward,
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
