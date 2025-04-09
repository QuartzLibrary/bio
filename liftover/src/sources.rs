use url::Url;

use utile::resource::{Compression, RawResource, UrlResource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsemblResource {
    pub key: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsemblHG {
    #[doc(alias = "hg19")]
    GRCh37,
    #[doc(alias = "hg38")]
    GRCh38,

    #[doc(alias = "Hg16")]
    NCBI34,
    #[doc(alias = "Hg17")]
    NCBI35,
    #[doc(alias = "Hg18")]
    NCBI36,
}
impl EnsemblResource {
    pub fn new(key: String) -> Self {
        Self { key }
    }
    pub fn new_human_liftover(from: EnsemblHG, to: EnsemblHG) -> Self {
        Self::new_human_liftover_raw(from.name(), to.name())
    }
    pub fn new_human_liftover_raw(from: &str, to: &str) -> Self {
        Self::new(format!(
            "assembly_mapping/homo_sapiens/{from}_to_{to}.chain.gz"
        ))
    }

    pub fn url(&self) -> Url {
        let key = &self.key;
        Url::parse(&format!("https://ftp.ensembl.org/pub/{key}")).unwrap()
    }

    fn url_resource(&self) -> UrlResource {
        UrlResource::new(self.url()).unwrap()
    }
}
impl RawResource for EnsemblResource {
    const NAMESPACE: &'static str = "ensembl";
    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<Compression> {
        if self.key.ends_with(".gz") || self.key.ends_with(".bgz") {
            Some(Compression::MultiGzip)
        } else {
            None
        }
    }

    type Reader = <UrlResource as RawResource>::Reader;
    fn size(&self) -> std::io::Result<u64> {
        self.url_resource().size()
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        self.url_resource().read()
    }

    type AsyncReader = <UrlResource as RawResource>::AsyncReader;
    async fn size_async(&self) -> std::io::Result<u64> {
        self.url_resource().size_async().await
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        self.url_resource().read_async().await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UcscResource {
    pub key: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UcscHG {
    Hg4,
    // Hg5, // Present but missing liftover files.
    // Hg6, // Present but missing liftover files.
    // Hg7, // Present but missing liftover files.
    // Hg8, // Present but missing liftover files.
    Hg10,
    Hg11,
    Hg12,
    Hg13,
    Hg15,

    #[doc(alias = "NCBI34")]
    Hg16,
    #[doc(alias = "NCBI35")]
    Hg17,
    #[doc(alias = "NCBI36")]
    Hg18,

    #[doc(alias = "GRCh38")]
    Hg38,

    Hg24,

    #[doc(alias = "GRCh37")]
    Hg19,

    #[doc(alias = "T2T-CHM13")]
    #[doc(alias = "T2T-CHM13v2.0")]
    Hs1,
}
impl UcscResource {
    pub fn new(key: String) -> Self {
        Self { key }
    }
    pub fn new_human_liftover(from: UcscHG, to: UcscHG) -> Self {
        Self::new_human_liftover_raw(from.name(), to.name_in_to_position())
    }
    pub fn new_human_liftover_raw(from: &str, to: &str) -> Self {
        Self::new(format!(
            "goldenPath/{from}/liftOver/{from}To{to}.over.chain.gz"
        ))
    }

    pub fn url(&self) -> Url {
        let key = &self.key;
        Url::parse(&format!("https://hgdownload2.cse.ucsc.edu/{key}")).unwrap()
    }

    fn url_resource(&self) -> UrlResource {
        UrlResource::new(self.url()).unwrap()
    }
}
impl RawResource for UcscResource {
    const NAMESPACE: &'static str = "ucsc";
    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<Compression> {
        if self.key.ends_with(".gz") || self.key.ends_with(".bgz") {
            Some(Compression::MultiGzip)
        } else {
            None
        }
    }

    type Reader = <UrlResource as RawResource>::Reader;
    fn size(&self) -> std::io::Result<u64> {
        self.url_resource().size()
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        self.url_resource().read()
    }

    type AsyncReader = <UrlResource as RawResource>::AsyncReader;
    async fn size_async(&self) -> std::io::Result<u64> {
        self.url_resource().size_async().await
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        self.url_resource().read_async().await
    }
}

mod boilerplate {
    use std::fmt;

    use super::{EnsemblHG, UcscHG};

    impl EnsemblHG {
        pub fn name(&self) -> &'static str {
            match self {
                Self::GRCh37 => "GRCh37",
                Self::GRCh38 => "GRCh38",
                Self::NCBI34 => "NCBI34",
                Self::NCBI35 => "NCBI35",
                Self::NCBI36 => "NCBI36",
            }
        }
        pub fn iter() -> impl Iterator<Item = Self> {
            [
                Self::GRCh37,
                Self::GRCh38,
                Self::NCBI34,
                Self::NCBI35,
                Self::NCBI36,
            ]
            .into_iter()
        }
        pub fn valid_pairs() -> impl Iterator<Item = (Self, Self)> {
            let mut pairs = vec![];
            for from in Self::iter() {
                for to in Self::iter() {
                    if from == to || Self::is_missing(from, to) {
                        continue;
                    }

                    pairs.push((from, to));
                }
            }
            pairs.into_iter()
        }
        /// Used to skip missing files during testing.
        pub fn is_missing(from: Self, to: Self) -> bool {
            match (from, to) {
                (
                    Self::NCBI34 | Self::NCBI35 | Self::NCBI36,
                    Self::NCBI34 | Self::NCBI35 | Self::NCBI36,
                ) => true,
                (_, _) => false,
            }
        }
    }
    impl AsRef<str> for EnsemblHG {
        fn as_ref(&self) -> &str {
            self.name()
        }
    }
    impl fmt::Display for EnsemblHG {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.name())
        }
    }

    impl UcscHG {
        pub fn name(self) -> &'static str {
            match self {
                Self::Hg4 => "hg4",
                Self::Hg10 => "hg10",
                Self::Hg11 => "hg11",
                Self::Hg12 => "hg12",
                Self::Hg13 => "hg13",
                // Note: the docs link to the '10april2003' folder instead of 'hg15'.
                // The only md5 hash that is present for the liftover files matches, so they probably just forgot to update from the temp folder?
                Self::Hg15 => "hg15",
                Self::Hg16 => "hg16",
                Self::Hg17 => "hg17",
                Self::Hg18 => "hg18",
                Self::Hg19 => "hg19",
                Self::Hg24 => "hg24",
                Self::Hg38 => "hg38",
                Self::Hs1 => "hs1",
            }
        }
        /// Seems like the second name is capitalized. "aaaToBbb.over.chain.gz"
        pub fn name_in_to_position(self) -> &'static str {
            match self {
                Self::Hg4 => "Hg4",
                Self::Hg10 => "Hg10",
                Self::Hg11 => "Hg11",
                Self::Hg12 => "Hg12",
                Self::Hg13 => "Hg13",
                Self::Hg15 => "Hg15",
                Self::Hg16 => "Hg16",
                Self::Hg17 => "Hg17",
                Self::Hg18 => "Hg18",
                Self::Hg19 => "Hg19",
                Self::Hg24 => "Hg24",
                Self::Hg38 => "Hg38",
                Self::Hs1 => "Hs1",
            }
        }
        pub fn iter() -> impl Iterator<Item = Self> {
            [
                Self::Hg4,
                Self::Hg10,
                Self::Hg11,
                Self::Hg12,
                Self::Hg13,
                Self::Hg15,
                Self::Hg16,
                Self::Hg17,
                Self::Hg18,
                Self::Hg19,
                Self::Hg24,
                Self::Hg38,
                Self::Hs1,
            ]
            .into_iter()
        }
        pub fn valid_pairs() -> impl Iterator<Item = (Self, Self)> {
            let mut pairs = vec![];
            for from in Self::iter() {
                for to in Self::iter() {
                    if from == to || Self::is_missing(from, to) || Self::has_negative_dt(from, to) {
                        continue;
                    }

                    pairs.push((from, to));
                }
            }
            pairs.into_iter()
        }
        /// Used to skip missing files during testing.
        pub fn is_missing(from: Self, to: Self) -> bool {
            match (from, to) {
                (
                    Self::Hg4,
                    Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg16
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg19
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,
                (
                    Self::Hg10,
                    Self::Hg4
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg19
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,
                (
                    Self::Hg11,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg16
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg12,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg19
                    | Self::Hg24
                    | Self::Hg38
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg13,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg19
                    | Self::Hg24
                    | Self::Hg38
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg15,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg18
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg16,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg17,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg18,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg16
                    | Self::Hg24
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg19,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg16
                    | Self::Hg24,
                ) => true,

                (
                    Self::Hg24,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg16
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg19
                    | Self::Hg38
                    | Self::Hs1,
                ) => true,

                (
                    Self::Hg38,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg16
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg24,
                ) => true,

                (
                    Self::Hs1,
                    Self::Hg4
                    | Self::Hg10
                    | Self::Hg11
                    | Self::Hg12
                    | Self::Hg13
                    | Self::Hg15
                    | Self::Hg16
                    | Self::Hg17
                    | Self::Hg18
                    | Self::Hg24,
                ) => true,
                (_, _) => false,
            }
        }
        /// Some liftover files have a negative `dt`, which is a bit sus.
        ///
        /// It could mean that the chain folds, but the UCSC liftover tool
        /// doesn't consistently behave as if that is the case.
        ///
        /// Given the design of the chain files, it seems possible that those files where
        /// generated incorrectly and no one noticed because it's a rarely used genome and
        /// the liftover files are old.
        pub fn has_negative_dt(from: Self, to: Self) -> bool {
            matches!((from, to), (Self::Hg12, Self::Hg15 | Self::Hg16))
        }
        pub fn is_available_in_online_interface(from: Self, to: Self) -> bool {
            match (from, to) {
                (Self::Hg16, Self::Hg17 | Self::Hg18 | Self::Hg19) => true,
                (Self::Hg17, Self::Hg15 | Self::Hg16 | Self::Hg18 | Self::Hg19) => true,
                (Self::Hg18, Self::Hg17 | Self::Hg19 | Self::Hg38) => true,
                (Self::Hg19, Self::Hg17 | Self::Hg18 | Self::Hg38 | Self::Hs1) => true, // Or "GCA_009914755.4"
                (Self::Hg38, Self::Hg19 | Self::Hs1) => true, // Or "GCA_009914755.4"
                (Self::Hs1, Self::Hg19 | Self::Hg38) => true,
                // ("GCA_009914755.4", Self::Hg19 | Self::Hg38) => true,
                (_, _) => false,
            }
        }
    }
    impl fmt::Display for UcscHG {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.name())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::Liftover;

    use super::*;

    #[test]
    fn test_ensembl_resource() {
        let resource = EnsemblResource::new_human_liftover(EnsemblHG::GRCh37, EnsemblHG::GRCh38);
        let liftover = Liftover::load(resource).unwrap();
        let from = liftover
            .chains
            .iter()
            .map(|c| &c.header.q.range.name)
            .collect::<BTreeSet<_>>();
        let to = liftover
            .chains
            .iter()
            .map(|c| &c.header.t.range.name)
            .collect::<BTreeSet<_>>();
        println!("\n{from:?}\n\n{to:?}");
    }
}
