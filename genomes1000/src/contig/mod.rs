use std::{
    borrow::Borrow,
    fmt::{self, Display},
    str::FromStr,
};

use serde::{de::Unexpected, Deserialize, Serialize};
use utile::io::FromUtf8Bytes;

use biocore::genome::Contig;

use crate::Sex;

mod grch37_meta;
mod grch38_meta;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GRCh38Contig {
    contig: &'static str,
}
impl GRCh38Contig {
    pub const CHR1: Self = Self { contig: "chr1" };
    pub const CHR2: Self = Self { contig: "chr2" };
    pub const CHR3: Self = Self { contig: "chr3" };
    pub const CHR4: Self = Self { contig: "chr4" };
    pub const CHR5: Self = Self { contig: "chr5" };
    pub const CHR6: Self = Self { contig: "chr6" };
    pub const CHR7: Self = Self { contig: "chr7" };
    pub const CHR8: Self = Self { contig: "chr8" };
    pub const CHR9: Self = Self { contig: "chr9" };
    pub const CHR10: Self = Self { contig: "chr10" };
    pub const CHR11: Self = Self { contig: "chr11" };
    pub const CHR12: Self = Self { contig: "chr12" };
    pub const CHR13: Self = Self { contig: "chr13" };
    pub const CHR14: Self = Self { contig: "chr14" };
    pub const CHR15: Self = Self { contig: "chr15" };
    pub const CHR16: Self = Self { contig: "chr16" };
    pub const CHR17: Self = Self { contig: "chr17" };
    pub const CHR18: Self = Self { contig: "chr18" };
    pub const CHR19: Self = Self { contig: "chr19" };
    pub const CHR20: Self = Self { contig: "chr20" };
    pub const CHR21: Self = Self { contig: "chr21" };
    pub const CHR22: Self = Self { contig: "chr22" };
    pub const MT: Self = Self { contig: "chrM" };
    pub const X: Self = Self { contig: "chrX" };
    pub const Y: Self = Self { contig: "chrY" };

    pub fn new(v: &str) -> Option<Self> {
        let contig = grch38_meta::META.get_entry(v)?.0;
        Some(Self { contig })
    }
    pub fn new_chr(number: usize) -> Option<Self> {
        Self::new(&format!("chr{number}"))
    }

    pub fn is_core(self) -> bool {
        pub static META: phf::Set<&'static str> = phf::phf_set! {
            "chr1",
            "chr2",
            "chr3",
            "chr4",
            "chr5",
            "chr6",
            "chr7",
            "chr8",
            "chr9",
            "chr10",
            "chr11",
            "chr12",
            "chr13",
            "chr14",
            "chr15",
            "chr16",
            "chr17",
            "chr18",
            "chr19",
            "chr20",
            "chr21",
            "chr22",
            "chrX",
            "chrY",
            "chrM",
        };
        META.contains(self.contig)
    }

    // Note that these are intentionally ordered as if they were strings,
    // to maintain maximum compatibility with string representations.
    pub const CHROMOSOMES: [Self; 25] = [
        Self::CHR1,
        Self::CHR10,
        Self::CHR11,
        Self::CHR12,
        Self::CHR13,
        Self::CHR14,
        Self::CHR15,
        Self::CHR16,
        Self::CHR17,
        Self::CHR18,
        Self::CHR19,
        Self::CHR2,
        Self::CHR20,
        Self::CHR21,
        Self::CHR22,
        Self::CHR3,
        Self::CHR4,
        Self::CHR5,
        Self::CHR6,
        Self::CHR7,
        Self::CHR8,
        Self::CHR9,
        Self::MT,
        Self::X,
        Self::Y,
    ];

    pub fn is_other(self) -> bool {
        !matches!(
            self,
            Self::CHR1
                | Self::CHR2
                | Self::CHR3
                | Self::CHR4
                | Self::CHR5
                | Self::CHR6
                | Self::CHR7
                | Self::CHR8
                | Self::CHR9
                | Self::CHR10
                | Self::CHR11
                | Self::CHR12
                | Self::CHR13
                | Self::CHR14
                | Self::CHR15
                | Self::CHR16
                | Self::CHR17
                | Self::CHR18
                | Self::CHR19
                | Self::CHR20
                | Self::CHR21
                | Self::CHR22
                | Self::X
                | Self::Y
        )
    }
    pub fn ploidy(self, sex: Sex) -> u8 {
        match (self, sex) {
            (Self::Y, Sex::Male) => 1,
            (Self::Y, Sex::Female) => 0,

            (Self::X, Sex::Male) => 1,
            (Self::X, Sex::Female) => 2,

            (Self::MT, _) => 1,

            _ => 2,
        }
    }
    pub fn is_diploid(self, sex: Sex) -> bool {
        self.ploidy(sex) == 2
    }
    pub fn is_haploid(self, sex: Sex) -> bool {
        self.ploidy(sex) == 1
    }

    fn new_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::new(bytes.as_ascii()?.as_str())
    }
}
impl Contig for GRCh38Contig {
    fn name(&self) -> &str {
        self.contig
    }
    fn size(&self) -> u64 {
        grch38_meta::META[self.contig].len
    }
}
impl Display for GRCh38Contig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contig)
    }
}
impl AsRef<str> for GRCh38Contig {
    fn as_ref(&self) -> &str {
        self.contig
    }
}
impl PartialEq<str> for GRCh38Contig {
    fn eq(&self, other: &str) -> bool {
        self.contig == other
    }
}
impl PartialEq<GRCh38Contig> for &str {
    fn eq(&self, other: &GRCh38Contig) -> bool {
        *self == other.contig
    }
}
impl PartialEq<String> for GRCh38Contig {
    fn eq(&self, other: &String) -> bool {
        self.contig == other
    }
}
impl PartialEq<GRCh38Contig> for String {
    fn eq(&self, other: &GRCh38Contig) -> bool {
        self == other.contig
    }
}
impl Borrow<str> for GRCh38Contig {
    fn borrow(&self) -> &str {
        self.contig
    }
}
impl FromStr for GRCh38Contig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or_else(|| s.to_owned())
    }
}
impl FromUtf8Bytes for GRCh38Contig {
    type Err = Result<String, std::string::FromUtf8Error>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        if let Some(contig) = Self::new_from_bytes(bytes) {
            Ok(contig)
        } else {
            Err(String::from_utf8(bytes.to_vec()))
        }
    }
}
impl Serialize for GRCh38Contig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.contig.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for GRCh38Contig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GRCh38Contig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                GRCh38Contig::new(v).ok_or(E::invalid_value(Unexpected::Str(v), &self))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GRCh37Contig {
    contig: &'static str,
}
impl GRCh37Contig {
    pub const CHR1: Self = Self { contig: "1" };
    pub const CHR2: Self = Self { contig: "2" };
    pub const CHR3: Self = Self { contig: "3" };
    pub const CHR4: Self = Self { contig: "4" };
    pub const CHR5: Self = Self { contig: "5" };
    pub const CHR6: Self = Self { contig: "6" };
    pub const CHR7: Self = Self { contig: "7" };
    pub const CHR8: Self = Self { contig: "8" };
    pub const CHR9: Self = Self { contig: "9" };
    pub const CHR10: Self = Self { contig: "10" };
    pub const CHR11: Self = Self { contig: "11" };
    pub const CHR12: Self = Self { contig: "12" };
    pub const CHR13: Self = Self { contig: "13" };
    pub const CHR14: Self = Self { contig: "14" };
    pub const CHR15: Self = Self { contig: "15" };
    pub const CHR16: Self = Self { contig: "16" };
    pub const CHR17: Self = Self { contig: "17" };
    pub const CHR18: Self = Self { contig: "18" };
    pub const CHR19: Self = Self { contig: "19" };
    pub const CHR20: Self = Self { contig: "20" };
    pub const CHR21: Self = Self { contig: "21" };
    pub const CHR22: Self = Self { contig: "22" };
    pub const MT: Self = Self { contig: "MT" };
    pub const X: Self = Self { contig: "X" };
    pub const Y: Self = Self { contig: "Y" };

    pub fn new(v: &str) -> Option<Self> {
        let contig = grch37_meta::META.get_entry(v)?.0;
        Some(Self { contig })
    }
    pub fn new_chr(number: usize) -> Option<Self> {
        Self::new(&format!("chr{number}"))
    }

    // Note that these are intentionally ordered as if they were strings,
    // to maintain maximum compatibility with string representations.
    pub const CHROMOSOMES: [Self; 25] = [
        Self::CHR1,
        Self::CHR10,
        Self::CHR11,
        Self::CHR12,
        Self::CHR13,
        Self::CHR14,
        Self::CHR15,
        Self::CHR16,
        Self::CHR17,
        Self::CHR18,
        Self::CHR19,
        Self::CHR2,
        Self::CHR20,
        Self::CHR21,
        Self::CHR22,
        Self::CHR3,
        Self::CHR4,
        Self::CHR5,
        Self::CHR6,
        Self::CHR7,
        Self::CHR8,
        Self::CHR9,
        Self::MT,
        Self::X,
        Self::Y,
    ];

    pub fn is_other(self) -> bool {
        !matches!(
            self,
            Self::CHR1
                | Self::CHR2
                | Self::CHR3
                | Self::CHR4
                | Self::CHR5
                | Self::CHR6
                | Self::CHR7
                | Self::CHR8
                | Self::CHR9
                | Self::CHR10
                | Self::CHR11
                | Self::CHR12
                | Self::CHR13
                | Self::CHR14
                | Self::CHR15
                | Self::CHR16
                | Self::CHR17
                | Self::CHR18
                | Self::CHR19
                | Self::CHR20
                | Self::CHR21
                | Self::CHR22
                | Self::X
                | Self::Y
        )
    }

    fn new_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::new(bytes.as_ascii()?.as_str())
    }
}
impl Contig for GRCh37Contig {
    fn name(&self) -> &str {
        self.contig
    }
    fn size(&self) -> u64 {
        grch37_meta::META[self.contig].len
    }
}
impl Display for GRCh37Contig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contig)
    }
}
impl AsRef<str> for GRCh37Contig {
    fn as_ref(&self) -> &str {
        self.contig
    }
}
impl PartialEq<str> for GRCh37Contig {
    fn eq(&self, other: &str) -> bool {
        self.contig == other
    }
}
impl PartialEq<GRCh37Contig> for &str {
    fn eq(&self, other: &GRCh37Contig) -> bool {
        *self == other.contig
    }
}
impl PartialEq<String> for GRCh37Contig {
    fn eq(&self, other: &String) -> bool {
        self.contig == other
    }
}
impl PartialEq<GRCh37Contig> for String {
    fn eq(&self, other: &GRCh37Contig) -> bool {
        self == other.contig
    }
}
impl Borrow<str> for GRCh37Contig {
    fn borrow(&self) -> &str {
        self.contig
    }
}
impl FromStr for GRCh37Contig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or_else(|| s.to_owned())
    }
}
impl FromUtf8Bytes for GRCh37Contig {
    type Err = Result<String, std::string::FromUtf8Error>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        if let Some(contig) = Self::new_from_bytes(bytes) {
            Ok(contig)
        } else {
            Err(String::from_utf8(bytes.to_vec()))
        }
    }
}
impl Serialize for GRCh37Contig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.contig.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for GRCh37Contig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GRCh37Contig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                GRCh37Contig::new(v).ok_or(E::invalid_value(Unexpected::Str(v), &self))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grch38_contig_chromosomes_sorted() {
        let names: Vec<&str> = GRCh38Contig::CHROMOSOMES.iter().map(|c| c.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(
            names, sorted,
            "GRCh38Contig::CHROMOSOMES must be lexicographically sorted"
        );
    }

    #[test]
    fn test_grch37_contig_chromosomes_sorted() {
        let names: Vec<&str> = GRCh37Contig::CHROMOSOMES.iter().map(|c| c.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(
            names, sorted,
            "GRCh37Contig::CHROMOSOMES must be lexicographically sorted"
        );
    }
}
