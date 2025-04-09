use std::{
    borrow::Borrow,
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut, Range},
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
impl<T> Sequence<T> {
    pub fn new(bases: Vec<T>) -> Self {
        Self { bases }
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

pub trait AsciiChar: Sized {
    // TODO: return &str instead
    fn encode(bases: &[Self]) -> String;

    type DecodeError;
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
