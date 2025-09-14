use core::str;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utile::value::enumerable::Enumerable;

use crate::sequence::{AsciiChar, Sequence, SequenceSlice};

pub type DnaSequence = Sequence<DnaBase>;
pub type AmbiguousDnaSequence = Sequence<AmbiguousDnaBase>;
pub type IupacDnaSequence = Sequence<IupacDnaBase>;

pub type DnaSequenceSlice = SequenceSlice<DnaBase>;
pub type AmbiguousDnaSequenceSlice = SequenceSlice<AmbiguousDnaBase>;
pub type IupacDnaSequenceSlice = SequenceSlice<IupacDnaBase>;

pub trait Complement {
    // Get the complementary nucleotide
    fn complement(self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum DnaBase {
    A = b'A',
    C = b'C',
    G = b'G',
    T = b'T',
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum MaybeDnaBase {
    A = b'A',
    C = b'C',
    G = b'G',
    T = b'T',
    N = b'N',
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum AmbiguousDnaBase {
    A = b'A',
    C = b'C',
    G = b'G',
    T = b'T',
    N = b'N',
}

/// https://www.bioinformatics.org/sms/iupac.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum IupacDnaBase {
    A = b'A',
    C = b'C',
    G = b'G',
    T = b'T',
    /// A or G (puRine)
    R = b'R',
    /// C or T (pYrimidine)
    Y = b'Y',
    /// G or C (Strong)
    S = b'S',
    /// A or T (Weak)
    W = b'W',
    /// G or T (Keto)
    K = b'K',
    /// A or C (aMino)
    M = b'M',
    /// C or G or T (not A)
    B = b'B',
    /// A or G or T (not C)
    D = b'D',
    /// A or C or T (not G)
    H = b'H',
    /// A or C or G (not T)
    V = b'V',
    N = b'N',
}

impl DnaBase {
    pub fn from_char(c: char) -> Option<Self> {
        Self::from_char_strict(c.to_ascii_uppercase())
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::from_byte_strict(b.to_ascii_uppercase())
    }

    pub fn from_char_strict(c: char) -> Option<Self> {
        match c {
            'A' => Some(Self::A),
            'C' => Some(Self::C),
            'G' => Some(Self::G),
            'T' => Some(Self::T),
            _ => None,
        }
    }
    pub fn from_byte_strict(b: u8) -> Option<Self> {
        match b {
            b'A' => Some(Self::A),
            b'C' => Some(Self::C),
            b'G' => Some(Self::G),
            b'T' => Some(Self::T),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::A => 'A',
            Self::C => 'C',
            Self::G => 'G',
            Self::T => 'T',
        }
    }
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::A => b'A',
            Self::C => b'C',
            Self::G => b'G',
            Self::T => b'T',
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::A, Self::C, Self::G, Self::T].into_iter()
    }

    // Check if the nucleotide is a purine (A or G)
    pub fn is_purine(&self) -> bool {
        matches!(self, Self::A | Self::G)
    }

    // Check if the nucleotide is a pyrimidine (C or T)
    pub fn is_pyrimidine(&self) -> bool {
        matches!(self, Self::C | Self::T)
    }
}
impl From<DnaBase> for u8 {
    fn from(value: DnaBase) -> Self {
        value.to_byte()
    }
}
impl Enumerable for DnaBase {
    const N: u128 = 4;
}
impl AsciiChar for DnaBase {
    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().map(Self::to_char).collect()
    }

    type DecodeError = DnaDecodeError;
    fn decode(mut bases: Vec<u8>) -> Result<Sequence<Self>, Self::DecodeError>
    where
        Self: Sized,
    {
        for (at, b) in bases.iter_mut().enumerate() {
            if Self::from_byte_strict(*b).is_none() {
                return Err(DnaDecodeError::InvalidSequence {
                    at,
                    byte: *b,
                    len: bases.len(),
                });
            }
        }
        bases.make_ascii_uppercase();
        Ok(Sequence::new(
            bases
                .into_iter()
                .map(Self::from_byte_strict)
                .map(Option::unwrap)
                .collect(),
        ))
    }
}

impl Complement for DnaBase {
    fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::C => Self::G,
            Self::G => Self::C,
        }
    }
}

// // Unsafe implementations as an exercise, this could make things faster,
// // but not worth it at the moment and I haven't looked deeply into correctness.
// // Hopefully safe transmute will be available soon.
// fn encode_unchecked_str(bases: &[DnaBase]) -> &str {
//     unsafe { std::str::from_utf8_unchecked(encode_unchecked(bases)) }
// }
// fn encode_unchecked(bases: &[DnaBase]) -> &[u8] {
//     let ptr_u8 = bases.as_ptr() as *const u8;
//     unsafe { std::slice::from_raw_parts(ptr_u8, bases.len()) }
// }
// fn decode_unchecked_str(s: String) -> Result<Vec<DnaBase>, String> {
//     match decode_unchecked(s.into_bytes()) {
//         Ok(bases) => Ok(bases),
//         Err(e) => Err(String::from_utf8(e).unwrap()),
//     }
// }
// fn decode_unchecked(mut bases: Vec<u8>) -> Result<Vec<DnaBase>, Vec<u8>> {
//     for b in &mut bases {
//         match *b {
//             b'A' | b'C' | b'G' | b'T' => {}
//             b'a' | b'c' | b'g' | b't' => *b = b.to_ascii_uppercase(),
//             _ => return Err(bases),
//         };
//     }
//     Ok({
//         // Ensure the original vector is not dropped.
//         let mut bases = std::mem::ManuallyDrop::new(bases);
//         let ptr = bases.as_mut_ptr() as *mut DnaBase;
//         unsafe { Vec::from_raw_parts(ptr, bases.len(), bases.capacity()) }
//     })
// }

impl std::fmt::Display for DnaBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
impl FromStr for DnaBase {
    type Err = DnaDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(DnaDecodeError::InvalidInputLength { from: s.to_owned() })
        } else {
            let char = s.chars().next().unwrap();
            Self::from_char(char).ok_or(DnaDecodeError::InvalidChar { from: char })
        }
    }
}

// MaybeDnaBase

impl MaybeDnaBase {
    pub fn from_char(c: char) -> Option<Self> {
        Self::from_char_strict(c.to_ascii_uppercase())
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::from_byte_strict(b.to_ascii_uppercase())
    }

    pub fn from_char_strict(c: char) -> Option<Self> {
        match c {
            'A' => Some(Self::A),
            'C' => Some(Self::C),
            'G' => Some(Self::G),
            'T' => Some(Self::T),
            'N' => Some(Self::N),
            _ => None,
        }
    }
    pub fn from_byte_strict(b: u8) -> Option<Self> {
        match b {
            b'A' => Some(Self::A),
            b'C' => Some(Self::C),
            b'G' => Some(Self::G),
            b'T' => Some(Self::T),
            b'N' => Some(Self::N),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::A => 'A',
            Self::C => 'C',
            Self::G => 'G',
            Self::T => 'T',
            Self::N => 'N',
        }
    }
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::A => b'A',
            Self::C => b'C',
            Self::G => b'G',
            Self::T => b'T',
            Self::N => b'N',
        }
    }

    // Check if the nucleotide is a purine (A or G)
    pub fn is_purine(&self) -> bool {
        matches!(self, Self::A | Self::G)
    }

    // Check if the nucleotide is a pyrimidine (C or T)
    pub fn is_pyrimidine(&self) -> bool {
        matches!(self, Self::C | Self::T)
    }

    pub fn is_ambiguous(&self) -> bool {
        !matches!(self, Self::A | Self::C | Self::G | Self::T)
    }
}
impl From<MaybeDnaBase> for u8 {
    fn from(value: MaybeDnaBase) -> Self {
        value.to_byte()
    }
}
impl AsciiChar for MaybeDnaBase {
    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().map(Self::to_char).collect()
    }

    type DecodeError = DnaDecodeError;
    fn decode(mut bases: Vec<u8>) -> Result<Sequence<Self>, Self::DecodeError>
    where
        Self: Sized,
    {
        for (at, b) in bases.iter_mut().enumerate() {
            if Self::from_byte_strict(*b).is_none() {
                return Err(DnaDecodeError::InvalidSequence {
                    at,
                    byte: *b,
                    len: bases.len(),
                });
            }
        }
        bases.make_ascii_uppercase();
        Ok(Sequence::new(
            bases
                .into_iter()
                .map(Self::from_byte_strict)
                .map(Option::unwrap)
                .collect(),
        ))
    }
}
impl Complement for MaybeDnaBase {
    fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::C => Self::G,
            Self::G => Self::C,
            Self::N => Self::N,
        }
    }
}

impl std::fmt::Display for MaybeDnaBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
impl FromStr for MaybeDnaBase {
    type Err = DnaDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(DnaDecodeError::InvalidInputLength { from: s.to_owned() })
        } else {
            let char = s.chars().next().unwrap();
            Self::from_char(char).ok_or(DnaDecodeError::InvalidChar { from: char })
        }
    }
}

// Ambiguous

impl AmbiguousDnaBase {
    pub fn from_char(c: char) -> Option<Self> {
        Self::from_char_strict(c.to_ascii_uppercase())
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::from_byte_strict(b.to_ascii_uppercase())
    }

    pub fn from_char_strict(c: char) -> Option<Self> {
        match c {
            'A' => Some(Self::A),
            'C' => Some(Self::C),
            'G' => Some(Self::G),
            'T' => Some(Self::T),
            'N' => Some(Self::N),
            _ => None,
        }
    }
    pub fn from_byte_strict(b: u8) -> Option<Self> {
        match b {
            b'A' => Some(Self::A),
            b'C' => Some(Self::C),
            b'G' => Some(Self::G),
            b'T' => Some(Self::T),
            b'N' => Some(Self::N),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::A => 'A',
            Self::C => 'C',
            Self::G => 'G',
            Self::T => 'T',
            Self::N => 'N',
        }
    }
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::A => b'A',
            Self::C => b'C',
            Self::G => b'G',
            Self::T => b'T',
            Self::N => b'N',
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::A, Self::C, Self::G, Self::T, Self::N].into_iter()
    }

    // Check if the nucleotide is a purine (A or G)
    pub fn is_purine(&self) -> bool {
        matches!(self, Self::A | Self::G)
    }
    // Check if the nucleotide is a pyrimidine (C or T)
    pub fn is_pyrimidine(&self) -> bool {
        matches!(self, Self::C | Self::T)
    }

    pub fn is_ambiguous(&self) -> bool {
        !matches!(self, Self::A | Self::C | Self::G | Self::T)
    }
}
impl From<AmbiguousDnaBase> for u8 {
    fn from(value: AmbiguousDnaBase) -> Self {
        value.to_byte()
    }
}
impl Enumerable for AmbiguousDnaBase {
    const N: u128 = 5;
}
impl AsciiChar for AmbiguousDnaBase {
    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().map(Self::to_char).collect()
    }

    type DecodeError = DnaDecodeError;
    fn decode(mut bases: Vec<u8>) -> Result<Sequence<Self>, Self::DecodeError>
    where
        Self: Sized,
    {
        for (at, b) in bases.iter_mut().enumerate() {
            if Self::from_byte_strict(*b).is_none() {
                return Err(DnaDecodeError::InvalidSequence {
                    at,
                    byte: *b,
                    len: bases.len(),
                });
            }
        }
        bases.make_ascii_uppercase();
        Ok(Sequence::new(
            bases
                .into_iter()
                .map(Self::from_byte_strict)
                .map(Option::unwrap)
                .collect(),
        ))
    }
}
impl Complement for AmbiguousDnaBase {
    fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::C => Self::G,
            Self::G => Self::C,
            Self::N => Self::N,
        }
    }
}

impl std::fmt::Display for AmbiguousDnaBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
impl FromStr for AmbiguousDnaBase {
    type Err = DnaDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(DnaDecodeError::InvalidInputLength { from: s.to_owned() })
        } else {
            let char = s.chars().next().unwrap();
            Self::from_char(char).ok_or(DnaDecodeError::InvalidChar { from: char })
        }
    }
}

// Iupac

impl IupacDnaBase {
    pub fn from_char(c: char) -> Option<Self> {
        Self::from_char_strict(c.to_ascii_uppercase())
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::from_byte_strict(b.to_ascii_uppercase())
    }

    pub fn from_char_strict(c: char) -> Option<Self> {
        match c {
            'A' => Some(Self::A),
            'C' => Some(Self::C),
            'G' => Some(Self::G),
            'T' => Some(Self::T),
            'R' => Some(Self::R),
            'Y' => Some(Self::Y),
            'S' => Some(Self::S),
            'W' => Some(Self::W),
            'K' => Some(Self::K),
            'M' => Some(Self::M),
            'B' => Some(Self::B),
            'D' => Some(Self::D),
            'H' => Some(Self::H),
            'V' => Some(Self::V),
            'N' => Some(Self::N),
            _ => None,
        }
    }
    pub fn from_byte_strict(b: u8) -> Option<Self> {
        match b {
            b'A' => Some(Self::A),
            b'C' => Some(Self::C),
            b'G' => Some(Self::G),
            b'T' => Some(Self::T),
            b'R' => Some(Self::R),
            b'Y' => Some(Self::Y),
            b'S' => Some(Self::S),
            b'W' => Some(Self::W),
            b'K' => Some(Self::K),
            b'M' => Some(Self::M),
            b'B' => Some(Self::B),
            b'D' => Some(Self::D),
            b'H' => Some(Self::H),
            b'V' => Some(Self::V),
            b'N' => Some(Self::N),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::A => 'A',
            Self::C => 'C',
            Self::G => 'G',
            Self::T => 'T',
            Self::R => 'R',
            Self::Y => 'Y',
            Self::S => 'S',
            Self::W => 'W',
            Self::K => 'K',
            Self::M => 'M',
            Self::B => 'B',
            Self::D => 'D',
            Self::H => 'H',
            Self::V => 'V',
            Self::N => 'N',
        }
    }
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::A => b'A',
            Self::C => b'C',
            Self::G => b'G',
            Self::T => b'T',
            Self::R => b'R',
            Self::Y => b'Y',
            Self::S => b'S',
            Self::W => b'W',
            Self::K => b'K',
            Self::M => b'M',
            Self::B => b'B',
            Self::D => b'D',
            Self::H => b'H',
            Self::V => b'V',
            Self::N => b'N',
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::A,
            Self::C,
            Self::G,
            Self::T,
            Self::R,
            Self::Y,
            Self::S,
            Self::W,
            Self::K,
            Self::M,
            Self::B,
            Self::D,
            Self::H,
            Self::V,
            Self::N,
        ]
        .into_iter()
    }

    pub fn is_purine(&self) -> bool {
        matches!(self, Self::A | Self::G | Self::R)
    }
    pub fn is_pyrimidine(&self) -> bool {
        matches!(self, Self::C | Self::T | Self::Y)
    }
    pub fn is_ambiguous(&self) -> bool {
        !matches!(self, Self::A | Self::C | Self::G | Self::T)
    }
}
impl From<IupacDnaBase> for u8 {
    fn from(value: IupacDnaBase) -> Self {
        value.to_byte()
    }
}
impl Enumerable for IupacDnaBase {
    const N: u128 = 15;
}
impl AsciiChar for IupacDnaBase {
    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().map(Self::to_char).collect()
    }

    type DecodeError = DnaDecodeError;
    fn decode(mut bases: Vec<u8>) -> Result<Sequence<Self>, Self::DecodeError>
    where
        Self: Sized,
    {
        for (at, b) in bases.iter_mut().enumerate() {
            if Self::from_byte_strict(*b).is_none() {
                return Err(DnaDecodeError::InvalidSequence {
                    at,
                    byte: *b,
                    len: bases.len(),
                });
            }
        }
        bases.make_ascii_uppercase();
        Ok(Sequence::new(
            bases
                .into_iter()
                .map(Self::from_byte_strict)
                .map(Option::unwrap)
                .collect(),
        ))
    }
}
impl Complement for IupacDnaBase {
    fn complement(self) -> Self {
        match self {
            Self::A => Self::T,
            Self::T => Self::A,
            Self::C => Self::G,
            Self::G => Self::C,
            Self::R => Self::Y, // A/G -> T/C
            Self::Y => Self::R, // C/T -> G/A
            Self::S => Self::S, // G/C -> C/G
            Self::W => Self::W, // A/T -> T/A
            Self::K => Self::M, // G/T -> C/A
            Self::M => Self::K, // A/C -> T/G
            Self::B => Self::V, // CGT -> GCA
            Self::D => Self::H, // AGT -> TCA
            Self::H => Self::D, // ACT -> TGA
            Self::V => Self::B, // ACG -> TGC
            Self::N => Self::N,
        }
    }
}

impl std::fmt::Display for IupacDnaBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl FromStr for IupacDnaBase {
    type Err = DnaDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(DnaDecodeError::InvalidInputLength { from: s.to_owned() })
        } else {
            let char = s.chars().next().unwrap();
            Self::from_char(char).ok_or(DnaDecodeError::InvalidChar { from: char })
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DnaDecodeError {
    #[error("Expected a single DNA base, got: {from}")]
    InvalidInputLength { from: String },
    #[error("Invalid DNA base: {from}")]
    InvalidByte { from: u8 },
    #[error("Invalid DNA base: {from}")]
    InvalidChar { from: char },
    #[error("Invalid DNA sequence: {byte:?} at {at}/{len} (invalid byte: {:?})", std::str::from_utf8(&[*byte]))]
    InvalidSequence { at: usize, byte: u8, len: usize },
}
impl From<DnaDecodeError> for std::io::Error {
    fn from(value: DnaDecodeError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}

mod eq {
    use super::*;

    impl PartialEq<IupacDnaBase> for DnaBase {
        fn eq(&self, other: &IupacDnaBase) -> bool {
            *self as u8 == *other as u8
        }
    }
    impl PartialEq<DnaBase> for IupacDnaBase {
        fn eq(&self, other: &DnaBase) -> bool {
            *self as u8 == *other as u8
        }
    }

    impl PartialEq<AmbiguousDnaBase> for DnaBase {
        fn eq(&self, other: &AmbiguousDnaBase) -> bool {
            *self as u8 == *other as u8
        }
    }
    impl PartialEq<DnaBase> for AmbiguousDnaBase {
        fn eq(&self, other: &DnaBase) -> bool {
            *self as u8 == *other as u8
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(DnaBase::iter().count() as u128, <DnaBase as Enumerable>::N);
        assert_eq!(
            AmbiguousDnaBase::iter().count() as u128,
            <AmbiguousDnaBase as Enumerable>::N
        );
        assert_eq!(
            IupacDnaBase::iter().count() as u128,
            <IupacDnaBase as Enumerable>::N
        );
    }
}
