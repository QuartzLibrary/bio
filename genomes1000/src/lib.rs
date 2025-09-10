#![feature(never_type)]
#![feature(ascii_char)]
#![feature(iterator_try_collect)]

mod genotype;
mod info;
mod parse;
mod slow;

pub mod contig;
pub mod pedigree;
pub mod resource;
pub mod simplified;

use either::Either;
use std::{cmp::Ordering, collections::BTreeMap, io};

use biocore::{
    dna::{DnaBase, DnaSequence},
    location::{ContigPosition, ContigRange},
    vcf::IndexedVcfReader,
};
use utile::{
    cache::FsCache,
    io::FromUtf8Bytes,
    iter::IteratorExt,
    resource::{RawResource, RawResourceExt},
};

use self::{pedigree::Pedigree, resource::Genomes1000Resource, simplified::SimplifiedRecord};

pub use self::{contig::GRCh38Contig, genotype::AltGenotype, info::RecordInfo};

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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Genotype {
    Missing,
    Haploid(HaploidGenotype),
    Diploid(DiploidGenotype),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct HaploidGenotype {
    pub value: u8,
}

/// Diploid/biallelic genotype/variant
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct DiploidGenotype {
    pub left: u8,
    pub phasing: GenotypePhasing,
    pub right: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
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
impl<S> Record<S> {
    pub fn at(&self) -> ContigPosition<GRCh38Contig> {
        ContigPosition {
            contig: self.contig,
            at: self.position - 1,
        }
    }
}
impl Record<Genotype> {
    /// Splits a multi-allelic variant into multiple bi-allelic variants.
    ///
    /// Note: invalidates info.
    pub fn split(self) -> impl Iterator<Item = Self> {
        if self.alternate_alleles.len() == 1 {
            return Either::Left([self].into_iter());
        }

        // log::info!(
        //     "[1000 Genomes][Split] Splitting {} alleles",
        //     self.alternate_alleles.len()
        // );

        Either::Right(self.alternate_alleles.clone().into_iter().enumerate().map(
            move |(i, alt)| {
                let mut record = self.clone();
                record.alternate_alleles = vec![alt];
                let i: u8 = i.try_into().unwrap();
                for sample in record.samples.iter_mut() {
                    Genotype::visit_values_mut(sample, |value| {
                        *value = if *value == i { 1 } else { 0 }
                    });
                }
                record
            },
        ))
    }
    /// Forces all alt alleles to be sequences.
    /// Returns [None] and clears its genotype back to reference if it fails to do so.
    ///
    /// NOTE: invalidates info.
    pub fn normalized(mut self) -> Option<Self> {
        let mut to_drop = vec![];
        for (i, alt) in self.alternate_alleles.iter_mut().enumerate() {
            if let AltGenotype::Sequence(_) = alt {
                continue;
            }

            if let Some(ref_allele) = self
                .reference_allele
                .iter()
                .copied()
                .try_collect::<DnaSequence>()
                && let Some(sequence) = alt.unpack(&ref_allele)
            {
                *alt = AltGenotype::Sequence(sequence);
                continue;
            };

            to_drop.push(i);
        }

        if to_drop.is_empty() {
            return Some(self);
        } else {
            self.info = "".to_owned();
        }

        // log::warn!(
        //     "[1000 Genomes][Normalize] Dropping {} alleles",
        //     to_drop.len()
        // );

        if self.alternate_alleles.len() == to_drop.len() {
            return None;
        }

        for i in to_drop.into_iter().rev() {
            self.alternate_alleles.remove(i);

            let i: u8 = i.try_into().unwrap();

            for sample in self.samples.iter_mut() {
                Genotype::visit_values_mut(sample, |value| match Ord::cmp(&*value, &i) {
                    Ordering::Less => {}
                    Ordering::Equal => *value = 0,
                    Ordering::Greater => *value -= 1,
                });
            }
        }

        Some(self)
    }
    pub fn simplified(self) -> Option<SimplifiedRecord> {
        let Some(reference_allele) = self
            .reference_allele
            .iter()
            .copied()
            .try_collect::<DnaSequence>()
        else {
            log::warn!("[1000 Genomes][Simplify] Incomplete reference allele");
            return None;
        };

        if self.alternate_alleles.len() != 1 {
            log::warn!(
                "[1000 Genomes][Simplify] Dropping {} alleles",
                self.alternate_alleles.len()
            );
            return None;
        }
        let alternate_allele = self.alternate_alleles.into_iter().next().unwrap();
        let AltGenotype::Sequence(alternate_allele) = alternate_allele else {
            log::warn!("[1000 Genomes][Simplify] Non-sequence alternate allele");
            return None;
        };

        Some(SimplifiedRecord {
            contig: self.contig,
            position: self.position,
            // id: self.id,
            reference_allele,
            alternate_allele,
            quality: self.quality,
            filter: self.filter,
            samples: self.samples,
        })
    }
}
impl Genotype {
    pub fn dosage(&self, variant: u8) -> u8 {
        let dose = |v: &u8| (*v == variant).into();
        match self {
            Genotype::Missing => 0,
            Genotype::Haploid(HaploidGenotype { value }) => dose(value),
            Genotype::Diploid(DiploidGenotype { left, right, .. }) => dose(left) + dose(right),
        }
    }
    pub fn ploidy(&self) -> Option<u8> {
        match self {
            Genotype::Missing => None,
            Genotype::Haploid(_) => Some(1),
            Genotype::Diploid(_) => Some(2),
        }
    }
    fn visit_values_mut(&mut self, mut f: impl FnMut(&mut u8)) {
        match self {
            Genotype::Missing => {}
            Genotype::Haploid(HaploidGenotype { value }) => f(value),
            Genotype::Diploid(DiploidGenotype { left, right, .. }) => {
                f(left);
                f(right);
            }
        }
    }
}

#[derive(Debug)]
pub struct Genomes1000Fs {
    sample_names: Vec<String>,
    pedigrees: BTreeMap<String, Pedigree>,
    readers: BTreeMap<GRCh38Contig, IndexedVcfReader<std::fs::File>>,
}

impl Genomes1000Fs {
    pub async fn new() -> io::Result<Self> {
        Self::new_with_cache(&FsCache::global()).await
    }
    pub fn sample_names(&self) -> &[String] {
        &self.sample_names
    }
    pub fn pedigree(&self, id: &str) -> Option<&Pedigree> {
        self.pedigrees.get(id)
    }
    pub async fn new_with_cache(cache: &FsCache) -> io::Result<Self> {
        let mut sample_names = None;

        let mut readers = BTreeMap::new();
        for contig in GRCh38Contig::CHROMOSOMES {
            let data = Genomes1000Resource::high_coverage_genotypes_contig_vcf(contig)
                .log_progress()
                .with_fs_cache(cache)
                .ensure_cached_async()
                .await?;

            let index = Genomes1000Resource::high_coverage_genotypes_contig_vcf_index(contig)
                .log_progress()
                .with_fs_cache(cache)
                .ensure_cached_async()
                .await?
                .decompressed();

            let reader = IndexedVcfReader::new(data.read()?, index.read()?)?;
            let names: Vec<_> = reader.header().sample_names().clone().into_iter().collect();

            if let Some(sample_names) = &sample_names {
                assert_eq!(sample_names, &names);
            } else {
                sample_names = Some(names);
            }

            readers.insert(contig, IndexedVcfReader::new(data.read()?, index.read()?)?);
        }

        let pedigrees = load_pedigree(
            Genomes1000Resource::high_coverage_pedigree()
                .log_progress()
                .with_fs_cache(cache)
                .ensure_cached_async()
                .await?,
        )
        .await?
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();

        Ok(Self {
            sample_names: sample_names.unwrap(),
            pedigrees,
            readers,
        })
    }
    pub fn query(
        &mut self,
        at: &ContigRange<GRCh38Contig>,
    ) -> io::Result<impl Iterator<Item = io::Result<Record<Genotype>>> + use<'_>> {
        let entry_c = if at.contig.is_core() {
            at.contig
        } else {
            GRCh38Contig::MT
        };

        let sample_count = self.sample_names.len();
        let reader = self.readers.get_mut(&entry_c).unwrap();

        let mut buf = vec![];
        let read_sample = sample_reading_function(entry_c);

        Ok(reader.query_raw(at)?.map(move |r| {
            let r = r?;
            Ok(parse::read_record(
                &mut buf,
                sample_count,
                &mut std::io::Cursor::new(r),
                read_sample,
            )?
            .unwrap())
        }))
    }
    pub fn query_simplified(
        &mut self,
        at: &ContigRange<GRCh38Contig>,
    ) -> io::Result<impl Iterator<Item = SimplifiedRecord> + use<'_>> {
        Ok(self
            .query(at)?
            .filter_map(|r: io::Result<Record<Genotype>>| match r {
                Ok(r) => Some(Ok(r.normalized()?)),
                Err(e) => Some(Err(e)),
            })
            .flat_map(|r: io::Result<Record<Genotype>>| match r {
                Ok(r) => Either::Left(r.split().map(Ok)),
                Err(e) => Either::Right([Err(e)].into_iter()),
            })
            .filter_map(|r: io::Result<Record<Genotype>>| match r {
                Ok(r) => Some(Ok(r.simplified()?)),
                Err(e) => Some(Err(e)),
            })
            .map(|r| r.unwrap()) // TODO
            .staged_sorted_by(simplified_stage_one, simplified_stage_two))
    }
}

pub async fn load_all_simplified() -> (Vec<String>, impl Iterator<Item = SimplifiedRecord>) {
    let (sample_names, variants) = load_all().await.unwrap();
    let sample_names: Vec<_> = sample_names.into_iter().map(|s| s.to_string()).collect();

    let variants = variants
        .map(|v| v.unwrap()) // TODO
        .filter_map(|v| v.normalized()) // Drops mutations we don't know the sequence of.
        .flat_map(|v| v.split()) // Splits multi-allelic variants into separate rows.
        .map(|v| v.simplified().unwrap()) // Cleaner simplified form given above. (TODO unwrap)
        .staged_sorted_by(simplified_stage_one, simplified_stage_two);
    (sample_names, variants)
}

pub async fn load_all() -> io::Result<(
    Vec<String>,
    impl Iterator<Item = io::Result<Record<Genotype>>>,
)> {
    let mut names = None;
    let mut chromosomes = vec![];
    for contig in GRCh38Contig::CHROMOSOMES {
        let (new_names, variants) = load_contig(contig).await?;
        chromosomes.push(variants);
        names = match names {
            Some(names) => {
                assert_eq!(names, new_names);
                Some(names)
            }
            None => Some(new_names),
        }
    }

    Ok((names.unwrap(), chromosomes.into_iter().flatten()))
}

pub async fn load_contig(
    c: GRCh38Contig,
) -> io::Result<(
    Vec<String>,
    impl Iterator<Item = io::Result<Record<Genotype>>>,
)> {
    let resource = Genomes1000Resource::high_coverage_genotypes_contig_vcf(c)
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?
        .decompressed()
        .buffered();

    parse::parse(resource.read()?, sample_reading_function(c))
}

pub async fn load_pedigree(resource: impl RawResource) -> io::Result<Vec<Pedigree>> {
    Ok(csv::ReaderBuilder::new()
        .delimiter(b' ')
        .from_reader(resource.read()?)
        .into_deserialize()
        .try_collect::<Vec<_>>()?
        .into_iter()
        .map(|mut p: Pedigree| {
            // "Based on these thresholds, two individuals, NA21310 and HG02300,
            // were listed as males, but had genotypes consistent with females"
            // https://www.biorxiv.org/content/10.1101/078600v1.full.pdf
            if p.id == "NA21310" || p.id == "HG02300" {
                p.sex = pedigree::Sex::Female;
            }
            p
        })
        .collect())
}

/// The fasta reader should be decompressed.
/// It should also implement [Seek](std::io::Seek) if random access is needed.
pub async fn load_grch38_reference_genome<F>(
    fasta: F,
    index: impl RawResource,
) -> io::Result<biocore::fasta::IndexedFastaReader<F::Reader>>
where
    F: RawResource,
    <F as RawResource>::Reader: std::io::BufRead,
{
    biocore::fasta::IndexedFastaReader::new(fasta.read()?, index.decompressed().buffered().read()?)
}

/// The fasta reader should be decompressed.
/// It should also implement [Seek](std::io::Seek) if random access is needed.
pub async fn load_grch37_reference_genome<F>(
    fasta: F,
    index: impl RawResource,
) -> io::Result<biocore::fasta::IndexedFastaReader<F::Reader>>
where
    F: RawResource,
    <F as RawResource>::Reader: std::io::BufRead,
{
    biocore::fasta::IndexedFastaReader::new(fasta.read()?, index.decompressed().buffered().read()?)
}

fn simplified_stage_one(a: &SimplifiedRecord, b: &SimplifiedRecord) -> Ordering {
    Ord::cmp(&a.at(), &b.at()).then_with(|| Ord::cmp(&a.reference_allele, &b.reference_allele))
}
fn simplified_stage_two(a: &SimplifiedRecord, b: &SimplifiedRecord) -> Ordering {
    Ord::cmp(&a.at(), &b.at())
        .then_with(|| Ord::cmp(&a.reference_allele, &b.reference_allele))
        .then_with(|| Ord::cmp(&a.alternate_allele, &b.alternate_allele))
}

fn sample_reading_function(contig: GRCh38Contig) -> fn(&[u8], &[u8]) -> io::Result<Genotype> {
    match contig {
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
        _ => |format, buf| ExtendedSample::<Genotype>::from_bytes(format, buf).map(Genotype::from),
    }
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
    impl fmt::Display for Genotype {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Missing => write!(f, "."),
                Self::Haploid(HaploidGenotype { value }) => write!(f, "{value}"),
                Self::Diploid(v) => {
                    write!(f, "{v}")
                }
            }
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
