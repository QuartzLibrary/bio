use std::{
    borrow::Borrow,
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut, Range, RangeBounds},
};

use ref_cast::RefCast;

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
    fn index(&self, index: usize) -> &Self::Output {
        &self.bases[index]
    }
}
impl<T> IndexMut<usize> for Sequence<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bases[index]
    }
}
impl<T> Index<Range<usize>> for Sequence<T> {
    type Output = SequenceSlice<T>;
    fn index(&self, index: Range<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<Range<usize>> for Sequence<T> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
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

    pub fn reverse_complement(self, f: impl Fn(T) -> T) -> Self {
        self.into_iter().rev().map(f).collect()
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
    fn index(&self, index: usize) -> &Self::Output {
        &self.bases[index]
    }
}
impl<T> IndexMut<usize> for SequenceSlice<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bases[index]
    }
}
impl<T> Index<Range<usize>> for SequenceSlice<T> {
    type Output = SequenceSlice<T>;
    fn index(&self, index: Range<usize>) -> &Self::Output {
        SequenceSlice::ref_cast(&self.bases[index])
    }
}
impl<T> IndexMut<Range<usize>> for SequenceSlice<T> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
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
