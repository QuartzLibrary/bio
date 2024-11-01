use std::{
    fmt,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Sequence<T> {
    bases: Vec<T>,
}
impl<T> Deref for Sequence<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.bases
    }
}
impl<T> DerefMut for Sequence<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bases
    }
}
impl<T> Sequence<T> {
    pub fn new(bases: Vec<T>) -> Self {
        Self { bases }
    }
    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
    }
}
impl<T: AsciiChar> Sequence<T> {
    // pub fn as_str(&self) -> &str {
    //     todo!()
    // }
}

impl<T: fmt::Display> fmt::Display for Sequence<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.bases {
            fmt::Display::fmt(b, f)?;
        }

        Ok(())
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
