use std::{collections::BTreeMap, io, ops::Index, sync::Arc};

use crate::{
    dna::DnaBase,
    location::{GenomePosition, GenomeRange},
    sequence::{Sequence, SequenceSlice},
};

pub trait Contig: AsRef<str> + PartialEq + Eq {
    fn size(&self) -> u64;
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
pub struct ArcContig {
    name: Arc<str>,
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
        Self { name, size }
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
    pub fn new(name: &'a str, size: u64) -> Self {
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
    pub fn get(&self, pos: &GenomePosition<C>) -> Option<B>
    where
        B: Clone,
    {
        self.contigs
            .get(&pos.name)?
            .get(usize::try_from(pos.at).unwrap())
            .cloned()
    }
    pub fn get_range(&self, range: &GenomeRange<C>) -> Option<&SequenceSlice<B>> {
        let at = usize::try_from(range.at.start).unwrap()..usize::try_from(range.at.end).unwrap();
        self.contigs.get(&range.name)?.get_range(at)
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
impl<C: Ord, B> Index<GenomePosition<C>> for InMemoryGenome<C, B> {
    type Output = B;
    #[track_caller]
    fn index(&self, index: GenomePosition<C>) -> &Self::Output {
        let at = usize::try_from(index.at).unwrap();
        &self[&index.name][at]
    }
}
impl<C: Ord, B> Index<GenomeRange<C>> for InMemoryGenome<C, B> {
    type Output = SequenceSlice<B>;
    #[track_caller]
    fn index(&self, index: GenomeRange<C>) -> &Self::Output {
        let at = usize::try_from(index.at.start).unwrap()..usize::try_from(index.at.end).unwrap();
        &self[&index.name][at]
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
