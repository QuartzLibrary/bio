use std::{
    borrow::Borrow,
    fmt,
    ops::{
        Deref, DerefMut, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive,
        RangeTo,
    },
    str::FromStr,
};

use ref_cast::RefCast;

use crate::dna::Complement;

/// A sequence of bases or proteins.
/// If this is a DNA/RNA sequence, it's 5' -> 3' by convention.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence<T> {
    bases: Vec<T>,
}
impl<T> Default for Sequence<T> {
    fn default() -> Self {
        Self { bases: vec![] }
    }
}
impl<T> Deref for Sequence<T> {
    type Target = SequenceSlice<T>;
    fn deref(&self) -> &Self::Target {
        SequenceSlice::ref_cast(&self.bases)
    }
}
impl<T> DerefMut for Sequence<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        SequenceSlice::ref_cast_mut(&mut self.bases)
    }
}
impl<T> Index<usize> for Sequence<T> {
    type Output = T;
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        &self.bases[index]
    }
}
impl<T> IndexMut<usize> for Sequence<T> {
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bases[index]
    }
}
impl<T> Index<Range<usize>> for Sequence<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<Range<usize>> for Sequence<T> {
    #[track_caller]
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeInclusive<usize>> for Sequence<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeInclusive<usize>> for Sequence<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeFrom<usize>> for Sequence<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeFrom<usize>> for Sequence<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeTo<usize>> for Sequence<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeTo<usize>> for Sequence<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeFull> for Sequence<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeFull) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeFull> for Sequence<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Borrow<SequenceSlice<T>> for Sequence<T> {
    fn borrow(&self) -> &SequenceSlice<T> {
        SequenceSlice::ref_cast(&self.bases)
    }
}

impl<T: fmt::Display> fmt::Display for Sequence<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.bases {
            fmt::Display::fmt(b, f)?;
        }

        Ok(())
    }
}
impl<T: AsciiChar> FromStr for Sequence<T> {
    type Err = T::DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::decode(s.as_bytes().to_vec())
    }
}
impl<T> IntoIterator for Sequence<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.bases.into_iter()
    }
}
impl<'a, T> IntoIterator for &'a Sequence<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.bases.iter()
    }
}
impl<T> FromIterator<T> for Sequence<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self::new(iter.into_iter().collect())
    }
}
impl<C: AsciiChar> TryFrom<noodles::fasta::record::Sequence> for Sequence<C> {
    type Error = C::DecodeError;
    fn try_from(sequence: noodles::fasta::record::Sequence) -> Result<Self, Self::Error> {
        // TODO: avoid re-allocating
        C::decode(sequence.as_ref().to_vec())
    }
}
impl<T> Sequence<T> {
    pub fn new(bases: Vec<T>) -> Self {
        Self { bases }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.bases.iter()
    }

    pub fn reverse_complement(self) -> Self
    where
        T: Complement,
    {
        self.bases.into_iter().rev().map(T::complement).collect()
    }

    pub fn spliced<R, I>(mut self, range: R, replace_with: I) -> Self
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        self.bases.splice(range, replace_with);
        self
    }
    pub fn splice<R, I>(&mut self, range: R, replace_with: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        self.bases.splice(range, replace_with).for_each(drop);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.bases.get(index)
    }
    pub fn get_range(&self, range: Range<usize>) -> Option<&SequenceSlice<T>> {
        self.bases.get(range).map(SequenceSlice::ref_cast)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.bases.append(&mut other.bases);
    }

    pub fn encode(&self) -> String
    where
        T: AsciiChar,
    {
        T::encode(&self.bases)
    }
    /// By convention this type is 5' -> 3', this encodes it in reverse
    /// to display it as 3' -> 5'.
    pub fn encode_3_to_5(self) -> String
    where
        T: AsciiChar,
    {
        T::encode(&self.bases.into_iter().rev().collect::<Vec<_>>())
    }

    pub fn contains(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.bases
            .windows(needle.len())
            .any(|window| window == needle)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, ref_cast::RefCast)]
#[repr(transparent)]
pub struct SequenceSlice<T> {
    bases: [T],
}
impl<T> Deref for SequenceSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.bases
    }
}
impl<T> DerefMut for SequenceSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bases
    }
}
impl<T> Index<usize> for SequenceSlice<T> {
    type Output = T;
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        &self.bases[index]
    }
}
impl<T> IndexMut<usize> for SequenceSlice<T> {
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bases[index]
    }
}
impl<T> Index<Range<usize>> for SequenceSlice<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<Range<usize>> for SequenceSlice<T> {
    #[track_caller]
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeInclusive<usize>> for SequenceSlice<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeInclusive<usize>> for SequenceSlice<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeFrom<usize>> for SequenceSlice<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeFrom<usize>> for SequenceSlice<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeTo<usize>> for SequenceSlice<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeTo<usize>> for SequenceSlice<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T> Index<RangeFull> for SequenceSlice<T> {
    type Output = SequenceSlice<T>;
    #[track_caller]
    fn index(&self, index: RangeFull) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<RangeFull> for SequenceSlice<T> {
    #[track_caller]
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        SequenceSlice::ref_cast_mut(&mut self.bases[index])
    }
}
impl<T: Clone> ToOwned for SequenceSlice<T> {
    type Owned = Sequence<T>;
    fn to_owned(&self) -> Self::Owned {
        Sequence::new(self.bases.to_vec())
    }
}
impl<T: fmt::Display> fmt::Display for SequenceSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self {
            fmt::Display::fmt(b, f)?;
        }

        Ok(())
    }
}
impl<'a, T> IntoIterator for &'a SequenceSlice<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.bases.iter()
    }
}
impl<T> SequenceSlice<T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        self.bases.get(index)
    }
    pub fn get_range(&self, range: Range<usize>) -> Option<&SequenceSlice<T>> {
        self.bases.get(range).map(SequenceSlice::ref_cast)
    }

    // TODO: return &str instead
    pub fn encode(&self) -> String
    where
        T: AsciiChar,
    {
        T::encode(&self.bases)
    }

    pub fn reverse_complement(&self) -> Sequence<T>
    where
        T: Complement + Clone,
    {
        self.into_iter().rev().cloned().map(T::complement).collect()
    }
}

pub trait AsciiChar: Sized {
    // TODO: return &str instead
    fn encode(bases: &[Self]) -> String;

    type DecodeError: Into<std::io::Error>;
    fn decode(bases: Vec<u8>) -> Result<Sequence<Self>, Self::DecodeError>;
}

impl<T: AsciiChar> serde::Serialize for Sequence<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&T::encode(&self.bases))
    }
}
impl<'de, T> serde::Deserialize<'de> for Sequence<T>
where
    T: AsciiChar,
    <T as AsciiChar>::DecodeError: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        T::decode(s.into_bytes()).map_err(serde::de::Error::custom)
    }
}
