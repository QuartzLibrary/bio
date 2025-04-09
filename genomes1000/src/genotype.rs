use core::fmt;
use std::str::FromStr;

use biocore::{
    dna::{DnaBase, DnaSequence},
    sequence::{AsciiChar, SequenceSlice},
};
use utile::io::FromUtf8Bytes;

// Alt genotypes actually seen in the data (besides 'Other'): {INS_ME_SVA, CN1, CN2, INS_ME_ALU, CN0, CN4, CN6, G, INS_MT, INS_ME_LINE1, CN3, INV, CN7, CN8, CN9, T, A, C, CN5}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AltGenotype {
    Sequence(DnaSequence),
    /// ##ALT=<ID=DEL,Description="Deletion">
    DEL,
    /// ##ALT=<ID=DUP,Description="Duplication">
    DUP,
    INS,
    /// ##ALT=<ID=INS:ME:ALU,Description="Insertion of ALU element">
    INS_ME_ALU,
    /// ##ALT=<ID=INS:ME:LINE1,Description="Insertion of LINE1 element">
    INS_ME_LINE1,
    /// ##ALT=<ID=INS:ME:SVA,Description="Insertion of SVA element">
    INS_ME_SVA,
    /// ##ALT=<ID=INS:MT,Description="Nuclear Mitochondrial Insertion">
    INS_MT,
    /// ##ALT=<ID=INV,Description="Inversion">
    INV,
    /// ##ALT=<ID=CN0,Description="Copy number allele: 0 copies">
    /// 0..=124
    CN(u8),
    Other(String),
}
impl fmt::Display for AltGenotype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Sequence(s) => return fmt::Display::fmt(s, f),
            Self::DEL => "<DEL>",
            Self::DUP => "<DUP>",
            Self::INS => "<INS>",
            Self::INS_ME_ALU => "<INS:ME:ALU>",
            Self::INS_ME_LINE1 => "<INS:ME:LINE1>",
            Self::INS_ME_SVA => "<INS:ME:SVA>",
            Self::INS_MT => "<INS:MT>",
            Self::INV => "<INV>",
            Self::CN(c) => {
                f.write_str("<CN")?;
                fmt::Display::fmt(c, f)?;
                f.write_str(">")?;

                return Ok(());
            }
            Self::Other(v) => v,
        };
        f.write_str(v)
    }
}
impl FromStr for AltGenotype {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            s if s
                .bytes()
                .all(|b| b == b'A' || b == b'C' || b == b'G' || b == b'T') =>
            {
                Self::Sequence(DnaBase::decode(s.to_owned().into_bytes())?)
            }
            "<DEL>" => Self::DEL,
            "<DUP>" => Self::DUP,
            "<INS>" => Self::INS,
            "<INS:ME:ALU>" => Self::INS_ME_ALU,
            "<INS:ME:LINE1>" => Self::INS_ME_LINE1,
            "<INS:ME:SVA>" => Self::INS_ME_SVA,
            "<INS:MT>" => Self::INS_MT,
            "<INV>" => Self::INV,
            _ => {
                fn parse_cn(s: &str) -> Option<u8> {
                    s.strip_prefix("<CN")?.strip_suffix(">")?.parse().ok()
                }

                if let Some(c) = parse_cn(s) {
                    Self::CN(c)
                } else {
                    Self::Other(s.to_owned())
                }
            }
        })
    }
}
impl FromUtf8Bytes for AltGenotype {
    type Err = std::io::Error;

    fn from_bytes(s: &[u8]) -> Result<Self, Self::Err> {
        Ok(match s {
            s if s
                .iter()
                .all(|&b| b == b'A' || b == b'C' || b == b'G' || b == b'T') =>
            {
                Self::Sequence(DnaBase::decode(s.to_vec())?)
            }
            b"<DEL>" => Self::DEL,
            b"<DUP>" => Self::DUP,
            b"<INS>" => Self::INS,
            b"<INS:ME:ALU>" => Self::INS_ME_ALU,
            b"<INS:ME:LINE1>" => Self::INS_ME_LINE1,
            b"<INS:ME:SVA>" => Self::INS_ME_SVA,
            b"<INS:MT>" => Self::INS_MT,
            b"<INV>" => Self::INV,
            _ => {
                fn parse_cn(s: &[u8]) -> Option<u8> {
                    u8::from_bytes(s.strip_prefix(b"<CN")?.strip_suffix(b">")?).ok()
                }

                if let Some(c) = parse_cn(s) {
                    Self::CN(c)
                } else {
                    Self::Other(String::from_utf8(s.to_vec()).map_err(utile::io::invalid_data)?)
                }
            }
        })
    }
}
impl AltGenotype {
    pub fn unpack(&self, reference: &SequenceSlice<DnaBase>) -> Option<DnaSequence> {
        Some(match self {
            AltGenotype::Sequence(sequence) => sequence.clone(),
            AltGenotype::DEL => DnaSequence::default(),
            AltGenotype::DUP => DnaSequence::new(
                reference
                    .iter()
                    .cloned()
                    .chain(reference.iter().cloned())
                    .collect(),
            ),
            AltGenotype::INS => return None, // Insertion of what?
            AltGenotype::INS_ME_ALU
            | AltGenotype::INS_ME_LINE1
            | AltGenotype::INS_ME_SVA
            | AltGenotype::INS_MT => return None, // TODO: what is the sequence?
            AltGenotype::INV => {
                let mut v = reference.to_vec();
                v.reverse();
                DnaSequence::new(v)
            }
            AltGenotype::CN(n) => DnaSequence::new(reference.repeat((*n).into())),
            AltGenotype::Other(v) => {
                log::warn!("Unpacking unknown alt genotype: {v}");
                return None;
            } // AltGenotype::Other(v) => todo!("{v}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::AltGenotype;

    #[tokio::test]
    #[ignore]
    async fn all_alt_genotypes() {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("reqwest", log::LevelFilter::Info)
            .filter_module("hyper_util", log::LevelFilter::Info)
            .init();

        let genotypes: HashSet<_> = crate::load_all()
            .await
            .unwrap()
            .flat_map(|r| r.unwrap().alternate_alleles)
            .filter(|g| !matches!(g, AltGenotype::Sequence(_)))
            .collect();

        println!("{genotypes:?}");
    }
}
