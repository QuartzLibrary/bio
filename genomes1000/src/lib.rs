#![feature(never_type)]

mod contig;
mod genotype;
mod info;
mod parse;
mod slow;

use std::io::BufReader;

use flate2::read::MultiGzDecoder;
use url::Url;
use utile::{
    cache::{Cache, CacheEntry, UrlEntry},
    io::FromUtf8Bytes,
};

use biocore::dna::DnaBase;

pub mod reference_genome;

pub use contig::B38Contig;
pub use genotype::AltGenotype;
pub use info::RecordInfo;

#[allow(dead_code)]
pub struct VcfFile<S> {
    format: String,
    file_data: jiff::civil::Date,
    reference: String,
    source: String,
    sample_names: Vec<String>,
    records: Vec<Record<S>>,
}

#[derive(Debug, Clone)]
pub struct Record<S> {
    pub contig: B38Contig,
    /// 1-based! 0 and n+1 means telomere (where n is length of contig).
    pub position: u64,
    pub id: String,
    pub reference_allele: Vec<Option<DnaBase>>,
    pub alternate_alleles: Vec<AltGenotype>,
    pub quality: Option<f64>,
    pub filter: String,
    pub info: String,
    pub format: String,
    pub samples: Vec<S>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Genotype {
    Missing,
    Haploid(HaploidGenotype),
    Diploid(DiploidGenotype),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HaploidGenotype {
    value: u8,
}

/// Diploid/biallelic genotype/variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiploidGenotype {
    pub left: u8,
    pub phasing: GenotypePhasing,
    pub right: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenotypePhasing {
    /// Marked with '|'
    Phased,
    /// Marked with '/'
    Unphased,
}

// Number=1: Exactly one value is present for the field.
// Number=0: The field is a flag, and it has no value. Its presence alone is enough to indicate something (usually true/false).
// Number=.: The number of values is variable. This is often used when the number of values depends on the context, such as multiple alternate alleles.
// Number=A: There is one value per alternate allele.
// Number=G: There is one value per possible genotype. This is commonly used for genotype likelihoods (e.g., PL field).
// Number=R: There is one value per allele, including the reference allele. This includes the reference allele plus all alternate alleles.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct ExtendedSample<GT> {
    /// ##FORMAT=<ID=AB,Number=1,Type=Float,Description="Allele balance for each het genotype">
    AB: Option<f64>,
    /// ##FORMAT=<ID=AD,Number=.,Type=Integer,Description="Allelic depths for the ref and alt alleles in the order listed">
    AD: Vec<u64>,
    /// ##FORMAT=<ID=DP,Number=1,Type=Integer,Description="Approximate read depth (reads with MQ=255 or with bad mates are filtered)">
    DP: Option<u64>,
    /// ##FORMAT=<ID=GQ,Number=1,Type=Integer,Description="Genotype Quality">
    GQ: Option<u64>,
    /// ##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">
    GT: Option<GT>,
    /// ##FORMAT=<ID=MIN_DP,Number=1,Type=Integer,Description="Minimum DP observed within the GVCF block">
    MIN_DP: Option<u64>,
    /// ##FORMAT=<ID=MQ0,Number=1,Type=Integer,Description="Number of Mapping Quality Zero Reads per sample">
    MQ0: Option<u64>,
    /// ##FORMAT=<ID=PGT,Number=1,Type=String,Description="Physical phasing haplotype information, describing how the alternate alleles are phased in relation to one another">
    PGT: Option<String>,
    /// ##FORMAT=<ID=PID,Number=1,Type=String,Description="Physical phasing ID information, where each unique ID within a given sample (but not across samples) connects records within a phasing group">
    PID: Option<String>,
    /// ##FORMAT=<ID=PL,Number=G,Type=Integer,Description="Normalized, Phred-scaled likelihoods for genotypes as defined in the VCF specification">
    PL: Option<Vec<u64>>,
    /// ##FORMAT=<ID=RGQ,Number=1,Type=Integer,Description="Unconditional reference genotype confidence, encoded as a phred quality -10*log10 p(genotype call is wrong)">
    RGQ: Option<u64>,
    /// ##FORMAT=<ID=SB,Number=4,Type=Integer,Description="Per-sample component statistics which comprise the Fisher's Exact Test to detect strand bias.">
    SB: Option<Vec<u64>>,
}

pub async fn load_all(
) -> Result<impl Iterator<Item = Result<Record<Genotype>, std::io::Error>>, std::io::Error> {
    let mut chromosomes = vec![];
    for i in 1..=22 {
        chromosomes.push(load_chr(i).await?);
    }
    let x = load_x().await?;
    let y = load_y().await?;
    let others = load_others().await?;

    Ok(chromosomes
        .into_iter()
        .flat_map(|i| i.map(|r| Ok(r?.map(Genotype::Diploid))))
        .chain(x)
        .chain(y.map(|r| Ok(r?.map(Genotype::from))))
        .chain(others.map(|r| Ok(r?.map(Genotype::from)))))
}

pub async fn load_contig(
    c: B38Contig,
) -> Result<Box<dyn Iterator<Item = Result<Record<Genotype>, std::io::Error>>>, std::io::Error> {
    Ok(match c {
        B38Contig::Chr1 => Box::new(load_chr(1).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr2 => Box::new(load_chr(2).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr3 => Box::new(load_chr(3).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr4 => Box::new(load_chr(4).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr5 => Box::new(load_chr(5).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr6 => Box::new(load_chr(6).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr7 => Box::new(load_chr(7).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr8 => Box::new(load_chr(8).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr9 => Box::new(load_chr(9).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr10 => Box::new(load_chr(10).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr11 => Box::new(load_chr(11).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr12 => Box::new(load_chr(12).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr13 => Box::new(load_chr(13).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr14 => Box::new(load_chr(14).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr15 => Box::new(load_chr(15).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr16 => Box::new(load_chr(16).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr17 => Box::new(load_chr(17).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr18 => Box::new(load_chr(18).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr19 => Box::new(load_chr(19).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr20 => Box::new(load_chr(20).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr21 => Box::new(load_chr(21).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::Chr22 => Box::new(load_chr(22).await?.map(|r| Ok(r?.map(Genotype::from)))),
        B38Contig::MT | B38Contig::Other(_) => {
            Box::new(load_others().await?.map(|r| Ok(r?.map(Genotype::from))))
        }
        B38Contig::X => Box::new(load_x().await?),
        B38Contig::Y => Box::new(load_y().await?.map(|r| Ok(r?.map(Genotype::from)))),
    })
}

pub async fn load_chr(
    number: usize,
) -> Result<impl Iterator<Item = Result<Record<DiploidGenotype>, std::io::Error>>, std::io::Error> {
    assert!((1..=22).contains(&number));
    let contig: B38Contig = format!("chr{number}").parse().unwrap();
    load_url(contig.download_url(), |_, buf| {
        DiploidGenotype::from_bytes(buf)
    })
    .await
}
pub async fn load_x(
) -> Result<impl Iterator<Item = Result<Record<Genotype>, std::io::Error>>, std::io::Error> {
    load_url(B38Contig::X.download_url(), |_, buf| {
        Genotype::from_bytes(buf)
    })
    .await
}
pub async fn load_y() -> Result<
    impl Iterator<Item = Result<Record<ExtendedSample<HaploidGenotype>>, std::io::Error>>,
    std::io::Error,
> {
    load_url(B38Contig::Y.download_url(), |format, buf| {
        ExtendedSample::from_bytes(format, buf)
    })
    .await
}
pub async fn load_others() -> Result<
    impl Iterator<Item = Result<Record<ExtendedSample<Genotype>>, std::io::Error>>,
    std::io::Error,
> {
    let url = contig::OTHER_DOWNLOAD_URL;
    load_url(url.parse().unwrap(), |format, buf| {
        ExtendedSample::from_bytes(format, buf)
    })
    .await
}

impl<S> Record<S> {
    fn map<O>(self, f: impl FnMut(S) -> O) -> Record<O> {
        Record {
            contig: self.contig,
            position: self.position,
            id: self.id,
            reference_allele: self.reference_allele,
            alternate_alleles: self.alternate_alleles,
            quality: self.quality,
            filter: self.filter,
            info: self.info,
            format: self.format,
            samples: self.samples.into_iter().map(f).collect(),
        }
    }
}

async fn load_url<S>(
    url: Url,
    read_sample: fn(&[u8], &[u8]) -> Result<S, std::io::Error>,
) -> Result<impl Iterator<Item = Result<Record<S>, std::io::Error>>, std::io::Error> {
    let fs_entry = download_and_cache_latest_contig(url).await?;

    let reader = BufReader::new(MultiGzDecoder::new(fs_entry.get()?));

    parse::parse(reader, read_sample)
}

async fn download_and_cache_latest_contig(url: Url) -> Result<CacheEntry, std::io::Error> {
    let fs_entry = Cache::global("1000genomes").entry(url.path().trim_start_matches('/'));

    UrlEntry::new(url)
        .unwrap()
        .get_and_cache_async("[Data][1000 Genomes]", fs_entry.clone())
        .await?;

    Ok(fs_entry)
}

mod boilerplate {
    use std::fmt;

    use super::{DiploidGenotype, ExtendedSample, Genotype, GenotypePhasing, HaploidGenotype};

    impl fmt::Display for GenotypePhasing {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    GenotypePhasing::Phased => '|',
                    GenotypePhasing::Unphased => '/',
                }
            )
        }
    }
    impl fmt::Display for DiploidGenotype {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                left,
                phasing,
                right,
            } = self;
            write!(f, "{left}{phasing}{right}")
        }
    }

    impl From<HaploidGenotype> for Genotype {
        fn from(value: HaploidGenotype) -> Self {
            Genotype::Haploid(value)
        }
    }
    impl From<DiploidGenotype> for Genotype {
        fn from(value: DiploidGenotype) -> Self {
            Genotype::Diploid(value)
        }
    }

    impl<GT> From<ExtendedSample<GT>> for Genotype
    where
        Genotype: From<GT>,
    {
        fn from(value: ExtendedSample<GT>) -> Self {
            match value.GT {
                Some(gt) => gt.into(),
                None => Genotype::Missing,
            }
        }
    }
}
