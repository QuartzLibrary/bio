use url::Url;
use utile::cache::{Cache, CacheEntry, UrlEntry};

use super::Liftover;

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
impl EnsemblHG {
    pub fn url(from: Self, to: Self) -> Url {
        ensembl_url(from.name(), to.name())
    }
    pub fn key(from: Self, to: Self) -> String {
        ensembl_key(from.name(), to.name())
    }
    pub fn global_cache(from: Self, to: Self) -> CacheEntry {
        Cache::global("ensembl").entry(Self::key(from, to))
    }
}
pub fn ensembl_url(from: &str, to: &str) -> Url {
    let key = ensembl_key(from, to);
    Url::parse(&format!("https://ftp.ensembl.org/pub/{key}")).unwrap()
}
pub fn ensembl_key(from: &str, to: &str) -> String {
    format!("assembly_mapping/homo_sapiens/{from}_to_{to}.chain.gz")
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
impl UcscHG {
    pub fn url(from: Self, to: Self) -> Url {
        ucsc_url(from.name(), to.name_in_to_position())
    }
    pub fn key(from: Self, to: Self) -> String {
        ucsc_key(from.name(), to.name_in_to_position())
    }
    pub fn global_cache(from: Self, to: Self) -> CacheEntry {
        Cache::global("ucsc").entry(Self::key(from, to))
    }
}
pub fn ucsc_url(from: &str, to: &str) -> Url {
    let key = ucsc_key(from, to);
    Url::parse(&format!("https://hgdownload2.cse.ucsc.edu/{key}")).unwrap()
}
pub fn ucsc_key(from: &str, to: &str) -> String {
    format!("goldenPath/{from}/liftOver/{from}To{to}.over.chain.gz")
}

impl Liftover {
    pub async fn load_human_ensembl(from: EnsemblHG, to: EnsemblHG) -> anyhow::Result<Self> {
        Self::load_ensembl(from.name(), to.name()).await
    }
    pub async fn load_ensembl(from: &str, to: &str) -> anyhow::Result<Self> {
        Ok(Self::read_gz_compressed(
            Self::cache_entry_ensembl(from, to).await?.get()?,
        )?)
    }

    pub async fn cache_human_entry_ensembl(
        from: EnsemblHG,
        to: EnsemblHG,
    ) -> anyhow::Result<CacheEntry> {
        Self::cache_entry_ensembl(from.name(), to.name()).await
    }
    pub async fn cache_entry_ensembl(from: &str, to: &str) -> anyhow::Result<CacheEntry> {
        if from == to {
            // TODO: return an identity and issue a warning instead.
            Err(anyhow::anyhow!(
                "Trying to create a liftover file for the same genome."
            ))?;
        }

        let fs_entry = Cache::global("ensembl").entry(ensembl_key(from, to));

        UrlEntry::new(ensembl_url(from, to))?
            .get_and_cache_async("[Data][Ensembl][liftover]", fs_entry.clone())
            .await?;

        Ok(fs_entry)
    }
}

impl Liftover {
    /// License: note that some UCSC liftover files do are not freely usable for commercial purposes.
    pub async fn load_human_ucsc(from: UcscHG, to: UcscHG) -> anyhow::Result<Self> {
        Self::load_ucsc(from.name(), to.name_in_to_position()).await
    }
    /// License: note that some UCSC liftover files do are not freely usable for commercial purposes.
    pub async fn load_ucsc(from: &str, to: &str) -> anyhow::Result<Self> {
        Ok(Self::read_gz_compressed(
            Self::cache_entry_ucsc(from, to).await?.get()?,
        )?)
    }

    /// License: note that some UCSC liftover files do are not freely usable for commercial purposes.
    pub async fn cache_human_entry_ucsc(from: UcscHG, to: UcscHG) -> anyhow::Result<CacheEntry> {
        Self::cache_entry_ucsc(from.name(), to.name()).await
    }
    /// License: note that some UCSC liftover files do are not freely usable for commercial purposes.
    pub async fn cache_entry_ucsc(from: &str, to: &str) -> anyhow::Result<CacheEntry> {
        if from == to {
            // TODO: return an identity and issue a warning instead.
            Err(anyhow::anyhow!(
                "Trying to create a liftover file for the same genome."
            ))?;
        }

        let fs_entry = Cache::global("ucsc").entry(ucsc_key(from, to));

        UrlEntry::new(ucsc_url(from, to))?
            .get_and_cache_async("[Data][UCSC][liftover]", fs_entry.clone())
            .await?;

        Ok(fs_entry)
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
