use serde::{Deserialize, Serialize};

use crate::sequence::Sequence;

pub type RnaSequence = Sequence<RnaBase>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum RnaBase {
    A = b'A',
    C = b'C',
    G = b'G',
    U = b'U',
}

impl RnaBase {
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'A' => Some(Self::A),
            'C' => Some(Self::C),
            'G' => Some(Self::G),
            'U' => Some(Self::U),
            _ => None,
        }
    }
    pub fn to_char(self) -> char {
        match self {
            Self::A => 'A',
            Self::C => 'C',
            Self::G => 'G',
            Self::U => 'U',
        }
    }

    /// Get the complementary nucleotide
    pub fn complement(self) -> Self {
        match self {
            Self::A => Self::U,
            Self::U => Self::A,
            Self::C => Self::G,
            Self::G => Self::C,
        }
    }

    /// Whether the nucleotide is a purine (A or G)
    pub fn is_purine(self) -> bool {
        matches!(self, Self::A | Self::G)
    }
    /// Whether the nucleotide is a pyrimidine (C or U)
    pub fn is_pyrimidine(self) -> bool {
        matches!(self, Self::C | Self::U)
    }
}

impl std::fmt::Display for RnaBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

mod random {
    use rand::Rng;

    use super::*;

    impl rand::distr::Distribution<RnaBase> for rand::distr::StandardUniform {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RnaBase {
            let _ = || match RnaBase::A {
                RnaBase::A | RnaBase::C | RnaBase::G | RnaBase::U => {
                    unreachable!("exhaustive match")
                }
            };
            match rng.random_range(0..4) {
                0 => RnaBase::A,
                1 => RnaBase::C,
                2 => RnaBase::G,
                3 => RnaBase::U,
                _ => unreachable!(),
            }
        }
    }
}
