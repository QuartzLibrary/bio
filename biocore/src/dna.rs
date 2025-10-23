use core::str;
use std::{ascii, str::FromStr};

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
    /// Y/S/K
    B = b'B',
    /// A or G or T (not C)
    /// R/W/K
    D = b'D',
    /// A or C or T (not G)
    /// Y/W/M
    H = b'H',
    /// A or C or G (not T)
    /// R/S/M
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

    pub fn to_char(self) -> char {
        self as u8 as char
    }
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::A, Self::C, Self::G, Self::T].into_iter()
    }

    // Check if the nucleotide is a purine (A or G)
    pub fn is_purine(self) -> bool {
        matches!(self, Self::A | Self::G)
    }

    // Check if the nucleotide is a pyrimidine (C or T)
    pub fn is_pyrimidine(self) -> bool {
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
    fn single_encode(self) -> ascii::Char {
        ascii::Char::from_u8(self.to_byte()).unwrap()
    }

    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().copied().map(Self::to_char).collect()
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

    pub fn to_char(self) -> char {
        self as u8 as char
    }
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::A, Self::C, Self::G, Self::T, Self::N].into_iter()
    }

    // Check if the nucleotide is a purine (A or G)
    pub fn is_purine(self) -> bool {
        matches!(self, Self::A | Self::G)
    }
    // Check if the nucleotide is a pyrimidine (C or T)
    pub fn is_pyrimidine(self) -> bool {
        matches!(self, Self::C | Self::T)
    }

    pub fn is_ambiguous(self) -> bool {
        self == Self::N
    }

    /// Returns all the variants that this base 'matches', excluding itself.
    fn iter_more_specific(self) -> impl Iterator<Item = Self> {
        use AmbiguousDnaBase::*;

        match self {
            A | C | G | T => [].iter(),
            N => [A, C, G, T].iter(),
        }
        .copied()
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
    fn single_encode(self) -> ascii::Char {
        ascii::Char::from_u8(self.to_byte()).unwrap()
    }

    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().copied().map(Self::to_char).collect()
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

    pub fn to_char(self) -> char {
        self as u8 as char
    }
    pub fn to_byte(self) -> u8 {
        self as u8
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

    pub fn is_purine(self) -> bool {
        matches!(self, Self::A | Self::G | Self::R)
    }
    pub fn is_pyrimidine(self) -> bool {
        matches!(self, Self::C | Self::T | Self::Y)
    }
    pub fn is_ambiguous(self) -> bool {
        !matches!(self, Self::A | Self::C | Self::G | Self::T)
    }

    /// Returns all the variants that this base 'matches', excluding itself.
    fn iter_more_specific(self) -> impl Iterator<Item = Self> {
        use IupacDnaBase::*;

        match self {
            A | C | G | T => [].iter(),

            R => [A, G].iter(),
            Y => [C, T].iter(),
            S => [G, C].iter(),
            W => [A, T].iter(),
            K => [G, T].iter(),
            M => [A, C].iter(),

            B => [C, T, G, Y, S, K].iter(),
            D => [A, T, G, R, W, K].iter(),
            H => [A, C, T, Y, W, M].iter(),
            V => [A, C, G, R, S, M].iter(),

            N => [A, C, G, T, R, Y, S, W, K, M, B, D, H, V].iter(),
        }
        .copied()
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
    fn single_encode(self) -> ascii::Char {
        ascii::Char::from_u8(self.to_byte()).unwrap()
    }

    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().copied().map(Self::to_char).collect()
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

pub mod pattern {
    use crate::sequence::MatchesChar;

    use super::*;

    impl MatchesChar<DnaBase> for DnaBase {
        fn iter_matching(self) -> impl Iterator<Item = DnaBase> {
            [self].into_iter()
        }
        fn matches(self, target: DnaBase) -> bool {
            self == target
        }
    }
    impl MatchesChar<AmbiguousDnaBase> for AmbiguousDnaBase {
        fn iter_matching(self) -> impl Iterator<Item = AmbiguousDnaBase> {
            self.iter_more_specific().chain([self])
        }
    }
    impl MatchesChar<IupacDnaBase> for IupacDnaBase {
        fn iter_matching(self) -> impl Iterator<Item = IupacDnaBase> {
            self.iter_more_specific().chain([self])
        }
    }

    impl MatchesChar<IupacDnaBase> for DnaBase {
        fn iter_matching(self) -> impl Iterator<Item = IupacDnaBase> {
            [self.into()].into_iter()
        }
        fn matches(self, target: IupacDnaBase) -> bool {
            IupacDnaBase::from(self) == target
        }
    }
    impl MatchesChar<AmbiguousDnaBase> for DnaBase {
        fn iter_matching(self) -> impl Iterator<Item = AmbiguousDnaBase> {
            [self.into()].into_iter()
        }
        fn matches(self, target: AmbiguousDnaBase) -> bool {
            AmbiguousDnaBase::from(self) == target
        }
    }

    impl MatchesChar<DnaBase> for IupacDnaBase {
        fn iter_matching(self) -> impl Iterator<Item = DnaBase> {
            match self {
                Self::A => [DnaBase::A].iter(),
                Self::C => [DnaBase::C].iter(),
                Self::G => [DnaBase::G].iter(),
                Self::T => [DnaBase::T].iter(),
                Self::R => [DnaBase::A, DnaBase::G].iter(),
                Self::Y => [DnaBase::C, DnaBase::T].iter(),
                Self::S => [DnaBase::G, DnaBase::C].iter(),
                Self::W => [DnaBase::A, DnaBase::T].iter(),
                Self::K => [DnaBase::G, DnaBase::T].iter(),
                Self::M => [DnaBase::A, DnaBase::C].iter(),
                Self::B => [DnaBase::C, DnaBase::T, DnaBase::G].iter(),
                Self::D => [DnaBase::A, DnaBase::T, DnaBase::G].iter(),
                Self::H => [DnaBase::A, DnaBase::C, DnaBase::T].iter(),
                Self::V => [DnaBase::A, DnaBase::C, DnaBase::G].iter(),
                Self::N => [DnaBase::A, DnaBase::C, DnaBase::G, DnaBase::T].iter(),
            }
            .copied()
        }
    }
    impl MatchesChar<DnaBase> for AmbiguousDnaBase {
        fn iter_matching(self) -> impl Iterator<Item = DnaBase> {
            match self {
                Self::A => [DnaBase::A].iter(),
                Self::C => [DnaBase::C].iter(),
                Self::G => [DnaBase::G].iter(),
                Self::T => [DnaBase::T].iter(),
                Self::N => [DnaBase::A, DnaBase::C, DnaBase::G, DnaBase::T].iter(),
            }
            .copied()
        }
        fn matches(self, target: DnaBase) -> bool {
            self.to_byte() == target.to_byte() || self == AmbiguousDnaBase::N
        }
    }

    impl MatchesChar<IupacDnaBase> for AmbiguousDnaBase {
        fn iter_matching(self) -> impl Iterator<Item = IupacDnaBase> {
            let iupac = IupacDnaBase::from(self);
            iupac.iter_more_specific().chain([iupac])
        }
    }
    impl MatchesChar<AmbiguousDnaBase> for IupacDnaBase {
        fn iter_matching(self) -> impl Iterator<Item = AmbiguousDnaBase> {
            let any = (self == Self::N).then_some(AmbiguousDnaBase::N);
            <IupacDnaBase as MatchesChar<DnaBase>>::iter_matching(self)
                .map(AmbiguousDnaBase::from)
                .chain(any)
        }
    }
}

pub mod util {
    use super::Complement;

    // TODO: Maybe move to an extension trait for arrays/sequences/slices?
    pub fn array_reverse_complement<const N: usize, T: Complement>(mut array: [T; N]) -> [T; N] {
        array.reverse();
        array.map(T::complement)
    }
}

mod from {
    use super::*;

    impl From<DnaBase> for AmbiguousDnaBase {
        fn from(value: DnaBase) -> Self {
            AmbiguousDnaBase::from_char(value.to_char()).unwrap()
        }
    }
    impl From<DnaBase> for IupacDnaBase {
        fn from(value: DnaBase) -> Self {
            IupacDnaBase::from_char(value.to_char()).unwrap()
        }
    }

    impl From<AmbiguousDnaBase> for IupacDnaBase {
        fn from(value: AmbiguousDnaBase) -> Self {
            IupacDnaBase::from_char(value.to_char()).unwrap()
        }
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
    use std::{collections::HashSet, fmt};

    use super::*;
    use crate::sequence::MatchesChar;

    #[test]
    fn sanity_checks() {
        assert_eq!(DnaBase::iter().count() as u128, <DnaBase as Enumerable>::N);
        assert_eq!(
            AmbiguousDnaBase::iter().count() as u128,
            <AmbiguousDnaBase as Enumerable>::N
        );
        assert_eq!(
            IupacDnaBase::iter().count() as u128,
            <IupacDnaBase as Enumerable>::N
        );

        assert_eq!(
            DnaBase::iter().count(),
            DnaBase::iter().collect::<HashSet<_>>().len()
        );
        assert_eq!(
            AmbiguousDnaBase::iter().count(),
            AmbiguousDnaBase::iter().collect::<HashSet<_>>().len()
        );
        assert_eq!(
            IupacDnaBase::iter().count(),
            IupacDnaBase::iter().collect::<HashSet<_>>().len()
        );
    }

    #[test]
    fn equivalence_checks() {
        #[track_caller]
        fn iter_matching_and_matches_equivalence<S, T>(a: S, b: T, is: Option<bool>)
        where
            S: MatchesChar<T> + fmt::Debug + Copy,
            T: PartialEq + fmt::Debug + Copy,
        {
            let matches = <S as MatchesChar<T>>::matches(a, b);
            let iter_matching = <S as MatchesChar<T>>::iter_matching(a).any(|m| m == b);
            assert_eq!(matches, iter_matching, "{a:?}/{b:?}");
            if let Some(is) = is {
                assert_eq!(is, matches, "wrong match: {a:?}/{b:?}");
            }
        }

        let bases = [
            (DnaBase::A, 1, 1, 1),
            (DnaBase::C, 1, 1, 1),
            (DnaBase::G, 1, 1, 1),
            (DnaBase::T, 1, 1, 1),
        ];
        assert_eq!(bases.len(), DnaBase::iter().count());
        for (a, count, count_amb, count_iupac) in bases {
            for b in DnaBase::iter() {
                let strict = Some(a == b);
                iter_matching_and_matches_equivalence(a, b, strict);
            }
            for b in AmbiguousDnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }
            for b in IupacDnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }

            assert_eq!(
                <DnaBase as MatchesChar<DnaBase>>::iter_matching(a).count(),
                count
            );
            assert_eq!(
                <DnaBase as MatchesChar<AmbiguousDnaBase>>::iter_matching(a).count(),
                count_amb
            );
            assert_eq!(
                <DnaBase as MatchesChar<IupacDnaBase>>::iter_matching(a).count(),
                count_iupac
            );
        }

        let bases_amb = [
            (AmbiguousDnaBase::A, 1, 1, 1),
            (AmbiguousDnaBase::C, 1, 1, 1),
            (AmbiguousDnaBase::G, 1, 1, 1),
            (AmbiguousDnaBase::T, 1, 1, 1),
            (AmbiguousDnaBase::N, 4, 5, 15),
        ];
        assert_eq!(bases_amb.len(), AmbiguousDnaBase::iter().count());
        for (a, count, count_amb, count_iupac) in bases_amb {
            for b in DnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }
            for b in AmbiguousDnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }
            for b in IupacDnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }

            assert_eq!(
                <AmbiguousDnaBase as MatchesChar<DnaBase>>::iter_matching(a).count(),
                count
            );
            assert_eq!(
                <AmbiguousDnaBase as MatchesChar<AmbiguousDnaBase>>::iter_matching(a).count(),
                count_amb
            );
            assert_eq!(
                <AmbiguousDnaBase as MatchesChar<IupacDnaBase>>::iter_matching(a).count(),
                count_iupac
            );
        }

        let bases_iupac = [
            (IupacDnaBase::A, 1, 1, 1),
            (IupacDnaBase::C, 1, 1, 1),
            (IupacDnaBase::G, 1, 1, 1),
            (IupacDnaBase::T, 1, 1, 1),
            //
            (IupacDnaBase::R, 2, 2, 3),
            (IupacDnaBase::Y, 2, 2, 3),
            (IupacDnaBase::S, 2, 2, 3),
            (IupacDnaBase::W, 2, 2, 3),
            (IupacDnaBase::K, 2, 2, 3),
            (IupacDnaBase::M, 2, 2, 3),
            //
            (IupacDnaBase::B, 3, 3, 7),
            (IupacDnaBase::D, 3, 3, 7),
            (IupacDnaBase::H, 3, 3, 7),
            (IupacDnaBase::V, 3, 3, 7),
            //
            (IupacDnaBase::N, 4, 5, 15),
        ];
        assert_eq!(bases_iupac.len(), IupacDnaBase::iter().count());
        for (a, count, count_amb, count_iupac) in bases_iupac {
            for b in DnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }
            for b in AmbiguousDnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }
            for b in IupacDnaBase::iter() {
                let strict = (a.to_byte() == b.to_byte()).then_some(true);
                iter_matching_and_matches_equivalence(a, b, strict);
            }

            assert_eq!(
                <IupacDnaBase as MatchesChar<DnaBase>>::iter_matching(a).count(),
                count
            );
            assert_eq!(
                <IupacDnaBase as MatchesChar<AmbiguousDnaBase>>::iter_matching(a).count(),
                count_amb
            );
            assert_eq!(
                <IupacDnaBase as MatchesChar<IupacDnaBase>>::iter_matching(a).count(),
                count_iupac
            );
        }
    }

    #[test]
    fn check_is_exact_match() {
        #[track_caller]
        fn check_match<A, B>(a: &str, b: &str, is_match: bool)
        where
            A: AsciiChar + Clone + fmt::Debug,
            B: AsciiChar + PartialEq + Clone + fmt::Debug,
            A: MatchesChar<B>,
        {
            let a: Sequence<A> = a.parse().map_err(drop).unwrap();
            let b: Sequence<B> = b.parse().map_err(drop).unwrap();
            assert_eq!(a.is_exact_match(&b), is_match, "{a:?}/{b:?}");
        }
        let base_base = [
            ("", "", true),
            ("", "A", false),
            ("A", "", false),
            ("A", "A", true),
            ("C", "C", true),
            ("G", "G", true),
            ("T", "T", true),
            ("ACGT", "ACGT", true),
            ("ACGT", "ACG", false),
            ("ACG", "ACGT", false),
            ("ACGT", "ACGA", false),
            ("AT", "TA", false),
        ];
        for (a, b, is_match) in base_base {
            check_match::<DnaBase, DnaBase>(a, b, is_match);
        }

        let base_amb = [
            ("", "", true),
            ("", "A", false),
            ("A", "", false),
            ("A", "A", true),
            ("C", "C", true),
            ("G", "G", true),
            ("T", "T", true),
            ("A", "N", false),
            ("CC", "CC", true),
            ("AC", "AN", false),
        ];
        for (a, b, is_match) in base_amb {
            check_match::<DnaBase, AmbiguousDnaBase>(a, b, is_match);
        }

        let base_iupac = [
            ("", "", true),
            ("", "A", false),
            ("A", "", false),
            ("A", "A", true),
            ("A", "R", false),
            ("G", "R", false),
            ("T", "W", false),
            ("C", "Y", false),
            ("AC", "AN", false),
            ("CC", "CC", true),
        ];
        for (a, b, is_match) in base_iupac {
            check_match::<DnaBase, IupacDnaBase>(a, b, is_match);
        }

        let amb_amb = [
            ("", "", true),
            ("", "N", false),
            ("N", "", false),
            ("A", "A", true),
            ("A", "N", false),
            ("N", "A", true),
            ("N", "N", true),
            ("N", "NN", false),
            ("NN", "N", false),
            ("NN", "AN", true),
        ];
        for (a, b, is_match) in amb_amb {
            check_match::<AmbiguousDnaBase, AmbiguousDnaBase>(a, b, is_match);
        }

        let amb_iupac = [
            ("", "", true),
            ("", "A", false),
            ("N", "", false),
            ("A", "A", true),
            ("A", "R", false),
            ("N", "R", true),
            ("N", "B", true),
            ("N", "N", true),
            ("N", "V", true),
            ("C", "M", false),
            ("AC", "AN", false),
        ];
        for (a, b, is_match) in amb_iupac {
            check_match::<AmbiguousDnaBase, IupacDnaBase>(a, b, is_match);
        }

        let iupac_iupac = [
            ("", "", true),
            ("", "N", false),
            ("N", "", false),
            ("A", "A", true),
            ("R", "A", true),
            ("R", "G", true),
            ("R", "R", true),
            ("R", "Y", false),
            ("B", "C", true),
            ("B", "Y", true),
            ("B", "R", false),
            ("N", "R", true),
            ("N", "B", true),
            ("HV", "HV", true),
            ("HV", "VH", false),
            ("R", "RR", false),
            ("RR", "R", false),
        ];
        for (a, b, is_match) in iupac_iupac {
            check_match::<IupacDnaBase, IupacDnaBase>(a, b, is_match);
        }
    }

    #[test]
    fn regex_pattern() {
        let base_base = [
            ("", ""),
            ("A", "[A]"),
            ("AT", "[A][T]"),
            ("ACGT", "[A][C][G][T]"),
            ("TTTT", "[T][T][T][T]"),
        ];
        for (a, b) in base_base {
            let a: Sequence<DnaBase> = a.parse().unwrap();
            assert_eq!(a.compile_regex::<DnaBase>(), b, "{a:?}/{b:?}");
        }

        let amb_amb = [
            ("", ""),
            ("N", "[ACGTN]"),
            ("AN", "[A][ACGTN]"),
            ("NA", "[ACGTN][A]"),
            ("NN", "[ACGTN][ACGTN]"),
            ("ACGNTA", "[A][C][G][ACGTN][T][A]"),
        ];
        for (a, b) in amb_amb {
            let a: Sequence<AmbiguousDnaBase> = a.parse().unwrap();
            assert_eq!(a.compile_regex::<AmbiguousDnaBase>(), b, "{a:?}/{b:?}");
        }

        let iupac_base = [
            ("", ""),
            ("A", "[A]"),
            ("C", "[C]"),
            ("G", "[G]"),
            ("T", "[T]"),
            ("R", "[AG]"),
            ("Y", "[CT]"),
            ("S", "[GC]"),
            ("W", "[AT]"),
            ("K", "[GT]"),
            ("M", "[AC]"),
            ("B", "[CTG]"),
            ("D", "[ATG]"),
            ("H", "[ACT]"),
            ("V", "[ACG]"),
            ("N", "[ACGT]"),
            ("AR", "[A][AG]"),
            ("RA", "[AG][A]"),
            ("RNY", "[AG][ACGT][CT]"),
            ("BHV", "[CTG][ACT][ACG]"),
            ("NN", "[ACGT][ACGT]"),
            ("RS", "[AG][GC]"),
            ("MK", "[AC][GT]"),
            ("SW", "[GC][AT]"),
            ("BDHVN", "[CTG][ATG][ACT][ACG][ACGT]"),
            ("ACGNTA", "[A][C][G][ACGT][T][A]"),
        ];
        for (a, b) in iupac_base {
            let a: Sequence<IupacDnaBase> = a.parse().unwrap();
            assert_eq!(a.compile_regex::<DnaBase>(), b, "{a:?}/{b:?}");
        }

        let iupac_iupac = [
            ("", ""),
            ("R", "[AGR]"),
            ("Y", "[CTY]"),
            ("S", "[GCS]"),
            ("W", "[ATW]"),
            ("K", "[GTK]"),
            ("M", "[ACM]"),
            ("B", "[CTGYSKB]"),
            ("D", "[ATGRWKD]"),
            ("H", "[ACTYWMH]"),
            ("V", "[ACGRSMV]"),
            ("N", "[ACGTRYSWKMBDHVN]"),
            ("AR", "[A][AGR]"),
            ("RA", "[AGR][A]"),
            ("RNY", "[AGR][ACGTRYSWKMBDHVN][CTY]"),
            ("BHV", "[CTGYSKB][ACTYWMH][ACGRSMV]"),
        ];
        for (a, b) in iupac_iupac {
            let a: Sequence<IupacDnaBase> = a.parse().unwrap();
            assert_eq!(a.compile_regex::<IupacDnaBase>(), b, "{a:?}/{b:?}");
        }
    }
}
