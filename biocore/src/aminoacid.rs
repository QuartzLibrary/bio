use std::{ascii, str::FromStr};

use serde::{Deserialize, Serialize};
use utile::value::enumerable::Enumerable;

use crate::sequence::{AsciiChar, Sequence, SequenceSlice};

pub type ProteinSequence = Sequence<AminoAcid>;
pub type ProteinSequenceSlice = SequenceSlice<AminoAcid>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum AminoAcid {
    /// Alanine (Ala)
    A = b'A',
    /// Arginine (Arg)
    R = b'R',
    /// Asparagine (Asn)
    N = b'N',
    /// Aspartate/Aspartic acid (Asp)
    D = b'D',
    /// Cysteine (Cys)
    C = b'C',
    /// Glutamine (Gln)
    Q = b'Q',
    /// Glutamate/Glutamic acid (Glu)
    E = b'E',
    /// Glycine (Gly)
    G = b'G',
    /// Histidine (His)
    H = b'H',
    /// Isoleucine (Ile)
    I = b'I',
    /// Leucine (Leu)
    L = b'L',
    /// Lysine (Lys)
    K = b'K',
    /// Methionine (Met)
    M = b'M',
    /// Phenylalanine (Phe)
    F = b'F',
    /// Proline (Pro)
    P = b'P',
    /// Serine (Ser)
    S = b'S',
    /// Threonine (Thr)
    T = b'T',
    /// Tryptophan (Trp)
    W = b'W',
    /// Tyrosine (Tyr)
    Y = b'Y',
    /// Valine (Val)
    V = b'V',
}
impl Enumerable for AminoAcid {
    const N: u128 = 20;
}
impl AminoAcid {
    pub fn from_char(c: char) -> Option<Self> {
        Self::from_char_strict(c.to_ascii_uppercase())
    }
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::from_byte_strict(b.to_ascii_uppercase())
    }
    pub fn from_char_strict(c: char) -> Option<Self> {
        match c {
            'A' => Some(Self::A),
            'R' => Some(Self::R),
            'N' => Some(Self::N),
            'D' => Some(Self::D),
            'C' => Some(Self::C),
            'Q' => Some(Self::Q),
            'E' => Some(Self::E),
            'G' => Some(Self::G),
            'H' => Some(Self::H),
            'I' => Some(Self::I),
            'L' => Some(Self::L),
            'K' => Some(Self::K),
            'M' => Some(Self::M),
            'F' => Some(Self::F),
            'P' => Some(Self::P),
            'S' => Some(Self::S),
            'T' => Some(Self::T),
            'W' => Some(Self::W),
            'Y' => Some(Self::Y),
            'V' => Some(Self::V),
            _ => None,
        }
    }
    pub fn from_byte_strict(b: u8) -> Option<Self> {
        match b {
            b'A' => Some(Self::A),
            b'R' => Some(Self::R),
            b'N' => Some(Self::N),
            b'D' => Some(Self::D),
            b'C' => Some(Self::C),
            b'Q' => Some(Self::Q),
            b'E' => Some(Self::E),
            b'G' => Some(Self::G),
            b'H' => Some(Self::H),
            b'I' => Some(Self::I),
            b'L' => Some(Self::L),
            b'K' => Some(Self::K),
            b'M' => Some(Self::M),
            b'F' => Some(Self::F),
            b'P' => Some(Self::P),
            b'S' => Some(Self::S),
            b'T' => Some(Self::T),
            b'W' => Some(Self::W),
            b'Y' => Some(Self::Y),
            b'V' => Some(Self::V),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::A => "Alanine",
            Self::R => "Arginine",
            Self::N => "Asparagine",
            Self::D => "Aspartate", // Aspartic acid
            Self::C => "Cysteine",
            Self::Q => "Glutamine",
            Self::E => "Glutamate", // Glutamic acid
            Self::G => "Glycine",
            Self::H => "Histidine",
            Self::I => "Isoleucine",
            Self::L => "Leucine",
            Self::K => "Lysine",
            Self::M => "Methionine",
            Self::F => "Phenylalanine",
            Self::P => "Proline",
            Self::S => "Serine",
            Self::T => "Threonine",
            Self::W => "Tryptophan",
            Self::Y => "Tyrosine",
            Self::V => "Valine",
        }
    }
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::A => "Ala",
            Self::R => "Arg",
            Self::N => "Asn",
            Self::D => "Asp",
            Self::C => "Cys",
            Self::Q => "Gln",
            Self::E => "Glu",
            Self::G => "Gly",
            Self::H => "His",
            Self::I => "Ile",
            Self::L => "Leu",
            Self::K => "Lys",
            Self::M => "Met",
            Self::F => "Phe",
            Self::P => "Pro",
            Self::S => "Ser",
            Self::T => "Thr",
            Self::W => "Trp",
            Self::Y => "Tyr",
            Self::V => "Val",
        }
    }

    pub fn to_char(&self) -> char {
        *self as u8 as char
    }
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::A,
            Self::R,
            Self::N,
            Self::D,
            Self::C,
            Self::Q,
            Self::E,
            Self::G,
            Self::H,
            Self::I,
            Self::L,
            Self::K,
            Self::M,
            Self::F,
            Self::P,
            Self::S,
            Self::T,
            Self::W,
            Self::Y,
            Self::V,
        ]
        .into_iter()
    }
}
impl From<AminoAcid> for u8 {
    fn from(value: AminoAcid) -> Self {
        value.to_byte()
    }
}

impl AsciiChar for AminoAcid {
    fn single_encode(self) -> ascii::Char {
        ascii::Char::from_u8(self.to_byte()).unwrap()
    }
    fn encode(bases: &[Self]) -> String
    where
        Self: Sized,
    {
        bases.iter().map(Self::to_char).collect()
    }

    type DecodeError = AminoDecodeError;
    fn decode(mut bases: Vec<u8>) -> Result<Sequence<Self>, Self::DecodeError>
    where
        Self: Sized,
    {
        for (at, b) in bases.iter_mut().enumerate() {
            if Self::from_byte_strict(*b).is_none() {
                return Err(AminoDecodeError::InvalidSequence {
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

impl std::fmt::Display for AminoAcid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl FromStr for AminoAcid {
    type Err = AminoDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(AminoDecodeError::InvalidInputLength { from: s.to_owned() })
        } else {
            let char = s.chars().next().unwrap();
            Self::from_char(char).ok_or(AminoDecodeError::InvalidChar { from: char })
        }
    }
}

pub mod codons {
    pub mod map {
        use std::{collections::HashMap, sync::LazyLock};

        use utile::collections::complete_map::CompleteHashMap;

        use crate::dna::{DnaBase, DnaBase::*};

        use super::super::AminoAcid;

        /// Standard genetic code translation from a DNA codon (5'->3') to an amino acid.
        /// (AA, start_codon, stop_codon)
        /// https://www.ncbi.nlm.nih.gov/Taxonomy/Utils/wprintgc.cgi?chapter=tgencodes
        /// https://web.archive.org/web/20250612231359/https://www.ncbi.nlm.nih.gov/Taxonomy/Utils/wprintgc.cgi?chapter=tgencodes
        pub static STANDARD: LazyLock<CompleteHashMap<[DnaBase; 3], Option<AminoAcid>>> =
            LazyLock::new(|| CompleteHashMap::new(ALL_STANDARD.into_iter().collect()));

        pub static STANDARD_REVERSE: LazyLock<
            CompleteHashMap<Option<AminoAcid>, Vec<[DnaBase; 3]>>,
        > = LazyLock::new(|| {
            let mut map: HashMap<Option<AminoAcid>, Vec<[DnaBase; 3]>> = HashMap::new();
            for (codon, aa) in STANDARD.iter() {
                map.entry(*aa).or_default().push(*codon);
            }
            CompleteHashMap::new(map)
        });

        pub static STANDARD_SYNONYMS: LazyLock<CompleteHashMap<[DnaBase; 3], Vec<[DnaBase; 3]>>> =
            LazyLock::new(|| {
                let mut map = HashMap::new();
                for (codon, aa) in STANDARD.iter() {
                    map.try_insert(
                        *codon,
                        STANDARD
                            .iter()
                            .filter(|(c, _)| *c != codon)
                            .filter(|(_, a)| *a == aa)
                            .map(|(c, _)| *c)
                            .collect(),
                    )
                    .unwrap();
                }
                CompleteHashMap::new(map)
            });

        const ALL_STANDARD: [([DnaBase; 3], Option<AminoAcid>); 64] = [
            ([T, T, T], Some(AminoAcid::F)), // Phe
            ([T, C, T], Some(AminoAcid::S)), // Ser
            ([T, A, T], Some(AminoAcid::Y)), // Tyr
            ([T, G, T], Some(AminoAcid::C)), // Cys
            ([T, T, C], Some(AminoAcid::F)), // Phe
            ([T, C, C], Some(AminoAcid::S)), // Ser
            ([T, A, C], Some(AminoAcid::Y)), // Tyr
            ([T, G, C], Some(AminoAcid::C)), // Cys
            ([T, T, A], Some(AminoAcid::L)), // Leu
            ([T, C, A], Some(AminoAcid::S)), // Ser
            ([T, A, A], None),               // Ter
            ([T, G, A], None),               // Ter
            ([T, T, G], Some(AminoAcid::L)), // Leu
            ([T, C, G], Some(AminoAcid::S)), // Ser
            ([T, A, G], None),               // Ter
            ([T, G, G], Some(AminoAcid::W)), // Trp
            ([C, T, T], Some(AminoAcid::L)), // Leu
            ([C, C, T], Some(AminoAcid::P)), // Pro
            ([C, A, T], Some(AminoAcid::H)), // His
            ([C, G, T], Some(AminoAcid::R)), // Arg
            ([C, T, C], Some(AminoAcid::L)), // Leu
            ([C, C, C], Some(AminoAcid::P)), // Pro
            ([C, A, C], Some(AminoAcid::H)), // His
            ([C, G, C], Some(AminoAcid::R)), // Arg
            ([C, T, A], Some(AminoAcid::L)), // Leu
            ([C, C, A], Some(AminoAcid::P)), // Pro
            ([C, A, A], Some(AminoAcid::Q)), // Gln
            ([C, G, A], Some(AminoAcid::R)), // Arg
            ([C, T, G], Some(AminoAcid::L)), // Leu
            ([C, C, G], Some(AminoAcid::P)), // Pro
            ([C, A, G], Some(AminoAcid::Q)), // Gln
            ([C, G, G], Some(AminoAcid::R)), // Arg
            ([A, T, T], Some(AminoAcid::I)), // Ile
            ([A, C, T], Some(AminoAcid::T)), // Thr
            ([A, A, T], Some(AminoAcid::N)), // Asn
            ([A, G, T], Some(AminoAcid::S)), // Ser
            ([A, T, C], Some(AminoAcid::I)), // Ile
            ([A, C, C], Some(AminoAcid::T)), // Thr
            ([A, A, C], Some(AminoAcid::N)), // Asn
            ([A, G, C], Some(AminoAcid::S)), // Ser
            ([A, T, A], Some(AminoAcid::I)), // Ile
            ([A, C, A], Some(AminoAcid::T)), // Thr
            ([A, A, A], Some(AminoAcid::K)), // Lys
            ([A, G, A], Some(AminoAcid::R)), // Arg
            ([A, T, G], Some(AminoAcid::M)), // Met
            ([A, C, G], Some(AminoAcid::T)), // Thr
            ([A, A, G], Some(AminoAcid::K)), // Lys
            ([A, G, G], Some(AminoAcid::R)), // Arg
            ([G, T, T], Some(AminoAcid::V)), // Val
            ([G, C, T], Some(AminoAcid::A)), // Ala
            ([G, A, T], Some(AminoAcid::D)), // Asp
            ([G, G, T], Some(AminoAcid::G)), // Gly
            ([G, T, C], Some(AminoAcid::V)), // Val
            ([G, C, C], Some(AminoAcid::A)), // Ala
            ([G, A, C], Some(AminoAcid::D)), // Asp
            ([G, G, C], Some(AminoAcid::G)), // Gly
            ([G, T, A], Some(AminoAcid::V)), // Val
            ([G, C, A], Some(AminoAcid::A)), // Ala
            ([G, A, A], Some(AminoAcid::E)), // Glu
            ([G, G, A], Some(AminoAcid::G)), // Gly
            ([G, T, G], Some(AminoAcid::V)), // Val
            ([G, C, G], Some(AminoAcid::A)), // Ala
            ([G, A, G], Some(AminoAcid::E)), // Glu
            ([G, G, G], Some(AminoAcid::G)), // Gly
        ];
    }

    pub mod usage {
        use std::sync::LazyLock;

        use utile::collections::complete_map::CompleteHashMap;

        use crate::dna::{DnaBase, DnaBase::*};

        /// Codon frequency and count in the human genome
        /// From https://dnahive.fda.gov/dna.cgi?cmd=codon_usage&id=537&mode=cocoputs
        /// "Homo sapiens (9606) Codon Usage Table"
        pub static GENOMIC_HUMAN: LazyLock<CompleteHashMap<[DnaBase; 3], f64>> =
            LazyLock::new(|| {
                CompleteHashMap::new(
                    ALL_GENOMIC_HUMAN
                        .map(|(codon, (freq, _count))| (codon, freq))
                        .into_iter()
                        .collect(),
                )
            });

        const ALL_GENOMIC_HUMAN: [([DnaBase; 3], (f64, u64)); 64] = [
            ([T, T, T], (17.14, 1385301)),
            ([T, C, T], (16.93, 1368632)),
            ([T, A, T], (12.11, 978774)),
            ([T, G, T], (10.40, 841042)),
            ([T, T, C], (17.48, 1413268)),
            ([T, C, C], (17.32, 1399962)),
            ([T, A, C], (13.49, 1090514)),
            ([T, G, C], (10.81, 873765)),
            ([T, T, A], (8.71, 703680)),
            ([T, C, A], (14.14, 1142684)),
            ([T, A, A], (0.44, 35218)),
            ([T, G, A], (0.79, 63801)),
            ([T, T, G], (13.44, 1086777)),
            ([T, C, G], (4.03, 325925)),
            ([T, A, G], (0.35, 28499)),
            ([T, G, G], (11.60, 937286)),
            ([C, T, T], (14.08, 1138433)),
            ([C, C, T], (19.31, 1560898)),
            ([C, A, T], (11.83, 956479)),
            ([C, G, T], (4.55, 367659)),
            ([C, T, C], (17.81, 1439345)),
            ([C, C, C], (19.11, 1544626)),
            ([C, A, C], (14.65, 1184041)),
            ([C, G, C], (8.71, 704401)),
            ([C, T, A], (7.44, 601662)),
            ([C, C, A], (18.92, 1529004)),
            ([C, A, A], (14.06, 1136523)),
            ([C, G, A], (6.42, 518818)),
            ([C, T, G], (36.10, 2918400)),
            ([C, C, G], (6.22, 503096)),
            ([C, A, G], (35.53, 2872161)),
            ([C, G, G], (10.79, 871786)),
            ([A, T, T], (16.48, 1331901)),
            ([A, C, T], (14.26, 1152700)),
            ([A, A, T], (18.43, 1489775)),
            ([A, G, T], (14.05, 1135376)),
            ([A, T, C], (18.67, 1508988)),
            ([A, C, C], (17.85, 1442511)),
            ([A, A, C], (18.30, 1478832)),
            ([A, G, C], (19.69, 1591829)),
            ([A, T, A], (8.08, 652939)),
            ([A, C, A], (16.52, 1335468)),
            ([A, A, A], (27.48, 2221062)),
            ([A, G, A], (13.28, 1073213)),
            ([A, T, G], (21.53, 1739992)),
            ([A, C, G], (5.59, 452037)),
            ([A, A, G], (31.77, 2567940)),
            ([A, G, G], (12.13, 980476)),
            ([G, T, T], (11.74, 949137)),
            ([G, C, T], (18.99, 1534685)),
            ([G, A, T], (24.03, 1942185)),
            ([G, G, T], (10.83, 875715)),
            ([G, T, C], (13.44, 1086717)),
            ([G, C, C], (25.84, 2088762)),
            ([G, A, C], (24.27, 1961667)),
            ([G, G, C], (19.79, 1599325)),
            ([G, T, A], (7.66, 618960)),
            ([G, C, A], (17.04, 1377145)),
            ([G, A, A], (33.65, 2719693)),
            ([G, G, A], (17.12, 1384137)),
            ([G, T, G], (25.87, 2090923)),
            ([G, C, G], (5.91, 477758)),
            ([G, A, G], (39.67, 3206546)),
            ([G, G, G], (15.35, 1240793)),
        ];
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AminoDecodeError {
    #[error("Expected a single amino acid, got: {from}")]
    InvalidInputLength { from: String },
    #[error("Invalid amino acid: {from}")]
    InvalidByte { from: u8 },
    #[error("Invalid amino acid: {from}")]
    InvalidChar { from: char },
    #[error("Invalid amino acid sequence: {byte:?} at {at}/{len} (invalid byte: {:?})", std::str::from_utf8(&[*byte]))]
    InvalidSequence { at: usize, byte: u8, len: usize },
    #[error("Invalid codon length: {len}, expected 3")]
    InvalidCodonLength { len: usize },
}
impl From<AminoDecodeError> for std::io::Error {
    fn from(value: AminoDecodeError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}

#[cfg(test)]
mod test {
    use utile::collections::complete_map::CompleteHashMap;

    use super::*;

    #[test]
    fn counts() {
        assert_eq!(
            AminoAcid::iter().count() as u128,
            <AminoAcid as Enumerable>::N
        );
    }

    #[test]
    fn complete_maps() {
        // The maps assert every element is present on creation.
        let _: &CompleteHashMap<_, _> = &*super::codons::map::STANDARD;
        let _: &CompleteHashMap<_, _> = &*super::codons::map::STANDARD_REVERSE;
        let _: &CompleteHashMap<_, _> = &*super::codons::map::STANDARD_SYNONYMS;
        let _: &CompleteHashMap<_, _> = &*super::codons::usage::GENOMIC_HUMAN;
    }
}
