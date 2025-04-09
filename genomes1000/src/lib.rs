#![feature(never_type)]
#![feature(ascii_char)]

mod contig;
mod genotype;
mod info;
mod parse;
mod slow;

use biocore::dna::DnaBase;
use resource::Genomes1000Resource;
use utile::{
    io::FromUtf8Bytes,
    resource::{RawResource, RawResourceExt},
};

pub mod resource;

pub use contig::GRCh38Contig;
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
    pub contig: GRCh38Contig,
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
    pub value: u8,
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
    for contig in GRCh38Contig::CHROMOSOMES {
        chromosomes.push(load_contig(contig).await?);
    }

    Ok(chromosomes.into_iter().flatten())
}

pub async fn load_contig(
    c: GRCh38Contig,
) -> Result<impl Iterator<Item = Result<Record<Genotype>, std::io::Error>>, std::io::Error> {
    let resource = Genomes1000Resource::high_coverage_genotypes_contig_vcf(c)
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?
        .decompressed()
        .buffered();

    parse::parse(
        resource.read()?,
        match c {
            GRCh38Contig::CHR1
            | GRCh38Contig::CHR2
            | GRCh38Contig::CHR3
            | GRCh38Contig::CHR4
            | GRCh38Contig::CHR5
            | GRCh38Contig::CHR6
            | GRCh38Contig::CHR7
            | GRCh38Contig::CHR8
            | GRCh38Contig::CHR9
            | GRCh38Contig::CHR10
            | GRCh38Contig::CHR11
            | GRCh38Contig::CHR12
            | GRCh38Contig::CHR13
            | GRCh38Contig::CHR14
            | GRCh38Contig::CHR15
            | GRCh38Contig::CHR16
            | GRCh38Contig::CHR17
            | GRCh38Contig::CHR18
            | GRCh38Contig::CHR19
            | GRCh38Contig::CHR20
            | GRCh38Contig::CHR21
            | GRCh38Contig::CHR22 => |_, buf| DiploidGenotype::from_bytes(buf).map(Genotype::from),
            GRCh38Contig::X => |_, buf| Genotype::from_bytes(buf),
            GRCh38Contig::Y => |format, buf| {
                ExtendedSample::<HaploidGenotype>::from_bytes(format, buf).map(Genotype::from)
            },
            _ => |format, buf| {
                ExtendedSample::<Genotype>::from_bytes(format, buf).map(Genotype::from)
            },
        },
    )
}

pub async fn load_grch38_reference_genome(
) -> std::io::Result<noodles::fasta::IndexedReader<std::io::BufReader<std::fs::File>>> {
    let resource = Genomes1000Resource::grch38_reference_genome()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;
    let index_resource = Genomes1000Resource::grch38_reference_genome_index()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;

    Ok(noodles::fasta::IndexedReader::new(
        resource.buffered().read()?,
        noodles::fasta::fai::Reader::new(index_resource.decompressed().buffered().read()?)
            .read_index()?,
    ))
}

pub async fn load_grch37_reference_genome(
) -> std::io::Result<noodles::fasta::IndexedReader<std::io::BufReader<std::fs::File>>> {
    let resource = Genomes1000Resource::old_grch37_reference_genome()
        .log_progress()
        .with_global_fs_cache()
        .decompressed() // Decompress *before* caching, so we have a file to index into.
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;
    let index_resource = Genomes1000Resource::old_grch37_reference_genome_index()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;

    Ok(noodles::fasta::IndexedReader::new(
        resource.buffered().read()?,
        noodles::fasta::fai::Reader::new(index_resource.decompressed().buffered().read()?)
            .read_index()?,
    ))
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
