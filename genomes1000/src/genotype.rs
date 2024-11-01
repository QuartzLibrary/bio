use core::fmt;
use std::str::FromStr;

use biocore::{
    dna::{DnaBase, DnaSequence},
    sequence::AsciiChar,
};

// Alt genotypes actually seen in the data (besides 'Other'): {INS_ME_SVA, CN1, CN2, INS_ME_ALU, CN0, CN4, CN6, G, INS_MT, INS_ME_LINE1, CN3, INV, CN7, CN8, CN9, T, A, C, CN5}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AltGenotype {
    Sequence(DnaSequence),
    /// ##ALT=<ID=CNV,Description="Copy Number Polymorphism">
    CNV,
    /// ##ALT=<ID=DEL,Description="Deletion">
    DEL,
    /// ##ALT=<ID=DUP,Description="Duplication">
    DUP,
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
            Self::CNV => "CNV",
            Self::DEL => "DEL",
            Self::DUP => "DUP",
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
        Self::alt_genotype_from_str(s)
    }
}

impl AltGenotype {
    fn alt_genotype_from_str(s: &str) -> Result<Self, std::io::Error> {
        Ok(match s {
            s if s
                .bytes()
                .all(|b| b == b'A' || b == b'C' || b == b'G' || b == b'T') =>
            {
                Self::Sequence(DnaBase::decode(s.to_owned().into_bytes())?)
            }
            "CNV" => Self::CNV,
            "DEL" => Self::DEL,
            "DUP" => Self::DUP,
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
