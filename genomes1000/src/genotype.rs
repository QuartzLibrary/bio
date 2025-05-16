use core::fmt;
use std::str::FromStr;

use biocore::{
    dna::{DnaBase, DnaSequence},
    sequence::{AsciiChar, SequenceSlice},
};
use utile::io::FromUtf8Bytes;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AltGenotype {
    Sequence(DnaSequence),
    DEL,
    DUP,
    INS,
    INV,
    /// Copy Number
    CN(u8),
    Other(&'static str),
    Unknown(String),
}
impl fmt::Display for AltGenotype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Sequence(s) => return fmt::Display::fmt(s, f),
            Self::DEL => "<DEL>",
            Self::DUP => "<DUP>",
            Self::INS => "<INS>",
            Self::INV => "<INV>",
            Self::CN(c) => {
                f.write_str("<CN")?;
                fmt::Display::fmt(c, f)?;
                f.write_str(">")?;

                return Ok(());
            }
            Self::Other(v) => v,
            Self::Unknown(v) => v,
        };
        f.write_str(v)
    }
}
impl FromStr for AltGenotype {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_cn(s: &str) -> Option<u8> {
            s.strip_prefix("<CN")?.strip_suffix(">")?.parse().ok()
        }

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
            "<INV>" => Self::INV,
            _ => {
                if let Some(c) = parse_cn(s) {
                    Self::CN(c)
                } else if let Some(v) = KNOWN.get_key(s) {
                    Self::Other(v)
                } else {
                    Self::Unknown(s.to_owned())
                }
            }
        })
    }
}
impl FromUtf8Bytes for AltGenotype {
    type Err = std::io::Error;

    fn from_bytes(s: &[u8]) -> Result<Self, Self::Err> {
        fn parse_cn(s: &[u8]) -> Option<u8> {
            u8::from_bytes(s.strip_prefix(b"<CN")?.strip_suffix(b">")?).ok()
        }

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
            b"<INV>" => Self::INV,
            _ => {
                if let Some(c) = parse_cn(s) {
                    Self::CN(c)
                } else {
                    let s = str::from_utf8(s).map_err(utile::io::invalid_data)?;
                    if let Some(v) = KNOWN.get_key(s) {
                        Self::Other(v)
                    } else {
                        Self::Unknown(s.to_owned())
                    }
                }
            }
        })
    }
}
impl AltGenotype {
    pub fn unpack(&self, reference: &SequenceSlice<DnaBase>) -> Option<DnaSequence> {
        Some(match self {
            Self::Sequence(sequence) => sequence.clone(),
            Self::DEL => DnaSequence::default(),
            Self::DUP => DnaSequence::new(
                reference
                    .iter()
                    .cloned()
                    .chain(reference.iter().cloned())
                    .collect(),
            ),
            Self::INS => return None, // Insertion of what?
            Self::INV => {
                let mut v = reference.to_vec();
                v.reverse();
                DnaSequence::new(v)
            }
            Self::CN(n) => DnaSequence::new(reference.repeat((*n).into())),
            Self::Other(v) if v.starts_with("<DEL:") => DnaSequence::default(),
            Self::Other(_) => return None,
            Self::Unknown(v) if v.starts_with("<DEL:") => DnaSequence::default(),
            Self::Unknown(v) => {
                if v != "*" {
                    log::warn!("[1000 Genomes] Unpacking unknown alt genotype: {v}");
                }
                return None;
            }
        })
    }
}

pub static KNOWN: phf::Set<&'static str> = phf::phf_set! {
    "*",
    "<DEL:ME:LINE|L1|L1HS>",
    "<DEL:ME:LINE|L1|L1MB8>",
    "<DEL:ME:LINE|L1|L1P1>",
    "<DEL:ME:LINE|L1|L1PA13>",
    "<DEL:ME:LINE|L1|L1PA2>",
    "<DEL:ME:LINE|L1|L1PA3>",
    "<DEL:ME:LINE|L1|L1PA5>",
    "<DEL:ME:LINE|L1|L1PA6>",
    "<DEL:ME:LINE|L1|L1PA7>",
    "<DEL:ME:LINE|L1|L1PA8>",
    "<DEL:ME:LINE|L1|L1PB1>",
    "<DEL:ME:Retroposon|SVA|SVA_C>",
    "<DEL:ME:Retroposon|SVA|SVA_D>",
    "<DEL:ME:Retroposon|SVA|SVA_E>",
    "<DEL:ME:Retroposon|SVA|SVA_F>",
    "<DEL:ME:SINE|Alu|Alu>",
    "<DEL:ME:SINE|Alu|AluJb>",
    "<DEL:ME:SINE|Alu|AluJr>",
    "<DEL:ME:SINE|Alu|AluSc>",
    "<DEL:ME:SINE|Alu|AluSc8>",
    "<DEL:ME:SINE|Alu|AluSg>",
    "<DEL:ME:SINE|Alu|AluSg4>",
    "<DEL:ME:SINE|Alu|AluSp>",
    "<DEL:ME:SINE|Alu|AluSq>",
    "<DEL:ME:SINE|Alu|AluSq10>",
    "<DEL:ME:SINE|Alu|AluSq2>",
    "<DEL:ME:SINE|Alu|AluSq4>",
    "<DEL:ME:SINE|Alu|AluSx>",
    "<DEL:ME:SINE|Alu|AluSx1>",
    "<DEL:ME:SINE|Alu|AluSx3>",
    "<DEL:ME:SINE|Alu|AluSx4>",
    "<DEL:ME:SINE|Alu|AluSz>",
    "<DEL:ME:SINE|Alu|AluSz6>",
    "<DEL:ME:SINE|Alu|AluY>",
    "<DEL:ME:SINE|Alu|AluYa5>",
    "<DEL:ME:SINE|Alu|AluYa8>",
    "<DEL:ME:SINE|Alu|AluYb8>",
    "<DEL:ME:SINE|Alu|AluYc3>",
    "<DEL:ME:SINE|Alu|AluYd8>",
    "<DEL:ME:SINE|Alu|AluYe5>",
    "<DEL:ME:SINE|Alu|AluYf1>",
    "<DEL:ME:SINE|Alu|AluYh3>",
    "<DEL:ME:SINE|Alu|AluYh7>",
    "<DEL:ME:SINE|Alu|AluYi6_4d>",
    "<DEL:ME:SINE|Alu|AluYi6>",
    "<DEL:ME:SINE|Alu|AluYj4>",
    "<DEL:ME:SINE|Alu|AluYk11>",
    "<DEL:ME:SINE|Alu|AluYk3>",
    "<DEL:ME:SINE|Alu|AluYk4>",
    "<DEL:ME:SINE|Alu|AluYm1>",
    "<DEL:ME>",
    "<DEL:xME:SINE|Alu|AluYb9>",
    "<DEL>",
    "<DUP>",
    "<INS:ME:ALU>",
    "<INS:ME:LINE1>",
    "<INS:ME:SVA>",
    "<INS:ME>",
    "<INS>",
    "<INV>",
};

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

        let (_sample_names, records) = crate::load_all().await.unwrap();

        let genotypes: HashSet<_> = records
            .flat_map(|r| r.unwrap().alternate_alleles)
            .filter(|g| !matches!(g, AltGenotype::Sequence(_)))
            .collect();

        println!("{genotypes:?}");
    }
}
