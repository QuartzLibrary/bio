use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt, io,
    ops::{Index, Range},
    sync::Arc,
};

use utile::{range::RangeExt, serde_ext::arc_str::SerdeArcStr};

use crate::{
    dna::DnaBase,
    location::{
        ContigPosition, ContigRange,
        orientation::{SequenceOrientation, WithOrientation},
    },
    sequence::{Sequence, SequenceSlice},
};

pub trait Contig: AsRef<str> + PartialEq + Eq + fmt::Debug {
    fn size(&self) -> u64;

    fn at(self, at: u64) -> ContigPosition<Self>
    where
        Self: Sized,
    {
        ContigPosition { contig: self, at }
    }
    fn at_range(self, at: Range<u64>) -> ContigRange<Self>
    where
        Self: Sized,
    {
        ContigRange { contig: self, at }
    }
}
impl<C> Contig for &C
where
    C: Contig + ?Sized,
{
    fn size(&self) -> u64 {
        <C as Contig>::size(self)
    }
}

/// A generic reference-counted contig that keeps track of its size.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct ArcContig {
    name: SerdeArcStr,
    size: u64,
}
impl AsRef<str> for ArcContig {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}
impl Contig for ArcContig {
    fn size(&self) -> u64 {
        self.size
    }
}
impl ArcContig {
    pub fn new(name: Arc<str>, size: u64) -> Self {
        Self {
            name: SerdeArcStr::new(name),
            size,
        }
    }
    pub fn from_contig(c: impl Contig) -> Self {
        Self::new(c.as_ref().into(), c.size())
    }
}

/// Useful as an interface any contig can be cast to without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContigRef<'a> {
    name: &'a str,
    size: u64,
}
impl<'a> AsRef<str> for ContigRef<'a> {
    fn as_ref(&self) -> &str {
        self.name
    }
}
impl<'a> Contig for ContigRef<'a> {
    fn size(&self) -> u64 {
        self.size
    }
}
impl<'a> ContigRef<'a> {
    pub const fn new(name: &'a str, size: u64) -> Self {
        Self { name, size }
    }
}
impl<'a, C: Contig> From<&'a C> for ContigRef<'a> {
    fn from(c: &'a C) -> Self {
        Self::new(c.as_ref(), c.size())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryGenome<C, B = DnaBase> {
    pub(crate) contigs: BTreeMap<C, Sequence<B>>,
}
impl<C: Ord, B> InMemoryGenome<C, B> {
    pub fn new(contigs: impl IntoIterator<Item = (C, Sequence<B>)>) -> Self {
        Self {
            contigs: contigs.into_iter().collect(),
        }
    }

    pub fn validate_sizes(&self) -> Result<(), ContigSizeError>
    where
        C: Contig,
    {
        for (c, s) in &self.contigs {
            let found_size = u64::try_from(s.len()).unwrap();
            let expected_size = c.size();
            if found_size != expected_size {
                Err(ContigSizeError {
                    contig: ArcContig::from_contig(c),
                    found_size,
                })?;
            }
        }
        Ok(())
    }
    pub fn get(&self, pos: &ContigPosition<C>) -> Option<B>
    where
        B: Clone,
    {
        self.contigs
            .get(&pos.contig)?
            .get(usize::try_from(pos.at).unwrap())
            .cloned()
    }
    pub fn get_range(&self, range: &ContigRange<C>) -> Option<&SequenceSlice<B>> {
        let at = usize::try_from(range.at.start).unwrap()..usize::try_from(range.at.end).unwrap();
        self.contigs.get(&range.contig)?.get_range(at)
    }

    pub fn contigs(&self) -> impl Iterator<Item = &C> {
        self.contigs.keys()
    }
}
impl<C: Ord, B> Index<C> for InMemoryGenome<C, B> {
    type Output = Sequence<B>;
    #[track_caller]
    fn index(&self, index: C) -> &Self::Output {
        &self[&index]
    }
}
impl<C: Ord, B> Index<&C> for InMemoryGenome<C, B> {
    type Output = Sequence<B>;
    #[track_caller]
    fn index(&self, index: &C) -> &Self::Output {
        &self.contigs[index]
    }
}
impl<C: Ord, B> Index<ContigPosition<C>> for InMemoryGenome<C, B> {
    type Output = B;
    #[track_caller]
    fn index(&self, index: ContigPosition<C>) -> &Self::Output {
        let at = usize::try_from(index.at).unwrap();
        &self[&index.contig][at]
    }
}
impl<C: Ord, B> Index<ContigRange<C>> for InMemoryGenome<C, B> {
    type Output = SequenceSlice<B>;
    #[track_caller]
    fn index(&self, index: ContigRange<C>) -> &Self::Output {
        let at = usize::try_from(index.at.start).unwrap()..usize::try_from(index.at.end).unwrap();
        &self[&index.contig][at]
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Expected contig {:?} to have size {}, got {found_size} instead.", contig.name, contig.size)]
pub struct ContigSizeError {
    pub contig: ArcContig,
    pub found_size: u64,
}
impl From<ContigSizeError> for io::Error {
    fn from(e: ContigSizeError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}

/// A normal contig, but with helpers to translate to the coordinates before/after an indel.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct EditedContig<C> {
    original: C,
    name: SerdeArcStr,
    remove: ContigRange<C>,
    insert: u64,
}
impl<C: Contig> Contig for EditedContig<C> {
    fn size(&self) -> u64 {
        self.original.size() - self.remove.len() + self.insert
    }
}
impl<C> AsRef<str> for EditedContig<C> {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}
impl<C: Contig> EditedContig<C> {
    pub fn new(contig: C, remove: ContigRange<C>, insert: u64) -> Self {
        let name = format!(
            "{} (D{}..{}I{})",
            contig.as_ref(),
            remove.at.start,
            remove.at.end,
            insert
        );
        Self {
            original: contig,
            name: SerdeArcStr::new(name),
            remove,
            insert,
        }
    }
    /// Takes a range on the original contig and returns the equivalent range on the edited contig.
    pub fn liftover(
        self,
        mut original: WithOrientation<ContigPosition<C>>,
    ) -> Option<WithOrientation<ContigPosition<Self>>> {
        let orientation = original.orientation;

        original.set_orientation(SequenceOrientation::Forward);

        let new = self.liftover_(original.v)?;

        let mut new = WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: new,
        };

        new.set_orientation(orientation);

        Some(new)
    }
    fn liftover_(self, p: ContigPosition<C>) -> Option<ContigPosition<Self>> {
        if p.contig != self.original {
            log::warn!(
                "Contig mismatch. Expected {:?}, got {:?}.",
                self.original,
                p.contig
            );
            return None;
        }
        let at = if self.remove.contains(&p) {
            // In the deleted region.
            log::warn!("Attempted to lift {p:?} over {self:?} but it is in the deleted region.");
            return None;
        } else if p.at < self.remove.at.start {
            // The range starts before the affected range, so we just return the original range.
            p.at
        } else {
            // The range starts after the affected range, so we adjust both the start and the end.
            (p - self.remove.len() + self.insert).at
        };

        Some(ContigPosition { contig: self, at })
    }
    /// Takes a range on the original contig and returns the equivalent range on the edited contig.
    pub fn liftover_range(
        self,
        mut original: WithOrientation<ContigRange<C>>,
    ) -> Option<WithOrientation<ContigRange<Self>>> {
        let orientation = original.orientation;

        original.set_orientation(SequenceOrientation::Forward);

        let new = self.liftover_range_(original.v)?;

        let mut new = WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: new,
        };

        new.set_orientation(orientation);

        Some(new)
    }
    fn liftover_range_(self, r: ContigRange<C>) -> Option<ContigRange<Self>> {
        if r.contig != self.original {
            log::warn!(
                "Contig mismatch. Expected {:?}, got {:?}.",
                self.original,
                r.contig
            );
            return None;
        }
        let at = if r.contains_range(&self.remove) {
            // The range fully contains the affected range, we just adjust the end.
            r.at.start..(r.at.end - self.remove.len() + self.insert)
        } else if !r.at.clone().intersection(self.remove.at.clone()).is_empty() {
            // There is a non-empty overlap, do not liftover.
            log::warn!("Attempted to lift {r:?} over {self:?} but there is a non-empty overlap.");
            return None;
        } else if r.at.start < self.remove.at.start {
            // The range starts before the affected range, so we just return the original range.
            r.at
        } else {
            // The range starts after the affected range, so we adjust both the start and the end.
            (r - self.remove.len() + self.insert).at
        };

        Some(ContigRange { contig: self, at })
    }
}
