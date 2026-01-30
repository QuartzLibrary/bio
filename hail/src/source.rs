use resource::{RawResource, UrlResource};
use url::Url;

const HAIL_COMMON_BUCKET: &str = "hail-common";

const GRCH38_REFERENCE_GENOME: &str = "references/Homo_sapiens_assembly38.fasta.gz";
const GRCH38_REFERENCE_GENOME_INDEX: &str = "references/Homo_sapiens_assembly38.fasta.fai";

const GRCH37_REFERENCE_GENOME: &str = "references/human_g1k_v37.fasta.gz";
const GRCH37_REFERENCE_GENOME_INDEX: &str = "references/human_g1k_v37.fasta.fai";

const GRCH37_TO_GRCH38_LIFTOVER_CHAIN: &str = "references/grch37_to_grch38.over.chain.gz";
const GRCH38_TO_GRCH37_LIFTOVER_CHAIN: &str = "references/grch38_to_grch37.over.chain.gz";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HailCommonResource {
    key: String,
}
impl HailCommonResource {
    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn grch38_reference_genome() -> Self {
        Self::new(GRCH38_REFERENCE_GENOME.to_owned())
    }
    pub fn grch38_reference_genome_index() -> Self {
        Self::new(GRCH38_REFERENCE_GENOME_INDEX.to_owned())
    }

    pub fn grch37_to_grch38_liftover_chain() -> Self {
        Self::new(GRCH37_TO_GRCH38_LIFTOVER_CHAIN.to_owned())
    }
    pub fn grch38_to_grch37_liftover_chain() -> Self {
        Self::new(GRCH38_TO_GRCH37_LIFTOVER_CHAIN.to_owned())
    }

    pub fn url(&self) -> Url {
        let key = &self.key;
        Url::parse(&format!("gs://{HAIL_COMMON_BUCKET}/{key}")).unwrap()
    }

    fn url_resource(&self) -> UrlResource {
        UrlResource::new(self.url()).unwrap()
    }
}
impl RawResource for HailCommonResource {
    const NAMESPACE: &'static str = "hail_common";

    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<resource::Compression> {
        match self.key.as_str() {
            GRCH38_REFERENCE_GENOME | GRCH37_REFERENCE_GENOME => {
                Some(resource::Compression::MultiGzip)
            }
            GRCH38_REFERENCE_GENOME_INDEX | GRCH37_REFERENCE_GENOME_INDEX => None,
            GRCH37_TO_GRCH38_LIFTOVER_CHAIN | GRCH38_TO_GRCH37_LIFTOVER_CHAIN => {
                Some(resource::Compression::Gzip)
            }
            _ => resource::Compression::infer_strict(&self.key),
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

mod old {
    use super::*;

    impl HailCommonResource {
        pub fn old_grch37_reference_genome() -> Self {
            Self::new(GRCH37_REFERENCE_GENOME.to_owned())
        }
        pub fn old_grch37_reference_genome_index() -> Self {
            Self::new(GRCH37_REFERENCE_GENOME_INDEX.to_owned())
        }
    }
}
