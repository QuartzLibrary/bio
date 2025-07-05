use url::Url;
use utile::resource::{RawResource, UrlResource};

const GRCH38_REFERENCE_GENOME_INDEXED: &str =
    "fasta/homo_sapiens/dna_index/Homo_sapiens.GRCh38.dna.toplevel.fa.gz";
const GRCH38_REFERENCE_GENOME_INDEXED_INDEX: &str =
    "fasta/homo_sapiens/dna_index/Homo_sapiens.GRCh38.dna.toplevel.fa.gz.fai";

const GRCH37_REFERENCE_GENOME: &str =
    "release-75/fasta/homo_sapiens/dna/Homo_sapiens.GRCh37.75.dna.primary_assembly.fa.gz";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnsemblResource {
    key: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssemblyMasking {
    None,
    Soft,
    Hard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssemblyKind {
    Primary,
    TopLevel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsemblHumanGenome {
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

    pub fn full_grch38_reference_genome() -> Self {
        Self::grch38_reference_genome_indexed(None)
    }
    pub fn full_grch38_reference_genome_index() -> Self {
        Self::grch38_reference_genome_indexed_index(None)
    }

    pub fn grch38_reference_genome(
        release: Option<u32>,
        kind: AssemblyKind,
        masking: AssemblyMasking,
    ) -> Self {
        let release = release
            .map(|r| r.to_string())
            .unwrap_or_else(|| "current".to_owned());
        let kind = match kind {
            AssemblyKind::Primary => "primary_assembly",
            AssemblyKind::TopLevel => "toplevel",
        };
        let masking = match masking {
            AssemblyMasking::None => "",
            AssemblyMasking::Soft => "_sm",
            AssemblyMasking::Hard => "_rm",
        };
        Self::new(format!(
            "{release}/fasta/homo_sapiens/dna/Homo_sapiens.GRCh38.dna{masking}.{kind}.fa.gz"
        ))
    }
    pub fn grch38_reference_genome_indexed(release: Option<u32>) -> Self {
        let release = release_str(release);
        Self::new(format!("{release}/{GRCH38_REFERENCE_GENOME_INDEXED}"))
    }
    pub fn grch38_reference_genome_indexed_index(release: Option<u32>) -> Self {
        let release = release_str(release);
        Self::new(format!("{release}/{GRCH38_REFERENCE_GENOME_INDEXED_INDEX}"))
    }

    pub fn grch37_to_grch38_mapping() -> Self {
        Self::assembly_mapping(None, EnsemblHumanGenome::GRCh37, EnsemblHumanGenome::GRCh38)
    }
    pub fn grch38_to_grch37_mapping() -> Self {
        Self::assembly_mapping(None, EnsemblHumanGenome::GRCh38, EnsemblHumanGenome::GRCh37)
    }
    pub fn assembly_mapping(
        release: Option<u32>,
        from: EnsemblHumanGenome,
        to: EnsemblHumanGenome,
    ) -> Self {
        let release = release_str(release);
        Self::new(format!(
            "{release}/assembly_chain/homo_sapiens/{from}_to_{to}.chain.gz"
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

    fn compression(&self) -> Option<utile::resource::Compression> {
        utile::resource::Compression::infer(&self.key)
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

fn release_str(release: Option<u32>) -> String {
    release
        .map(|r| format!("release-{r}"))
        .unwrap_or_else(|| "current".to_owned())
}

mod old {
    use super::*;

    impl EnsemblResource {
        pub fn old_grch37_reference_genome() -> Self {
            Self::new(GRCH37_REFERENCE_GENOME.to_owned())
        }
    }
}

mod boilerplate {
    use std::fmt;

    use super::*;

    impl EnsemblHumanGenome {
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
    impl AsRef<str> for EnsemblHumanGenome {
        fn as_ref(&self) -> &str {
            self.name()
        }
    }
    impl fmt::Display for EnsemblHumanGenome {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.name())
        }
    }
}
