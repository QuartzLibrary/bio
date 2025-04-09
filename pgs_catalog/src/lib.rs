#![feature(iterator_try_collect)]

use std::{
    io::{BufReader, Read},
    path::PathBuf,
};

use biocore::dna::DnaSequence;
use flate2::read::MultiGzDecoder;
use ordered_float::NotNan;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;
use utile::cache::{Cache, CacheEntry, UrlEntry};

pub mod metadata;
pub use ids::{pgs::PgsId, rs::RsId};

/// See [docs::HEADER_DOCS] and [docs::EXAMPLE_HEADER] and [docs::HARMONIZATION_EXTENSION].
/// Harmonization happens to a [GenomeBuild].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarmonizedStudy {
    /// Version of the scoring file format, e.g. '2.0'.
    format_version: String,
    pgs_info: PgsInfo,
    source_info: SourceInfo,
    harmonization_info: HarmonizationInfo,
    associations: Vec<HarmonizedStudyAssociation>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenomeBuild {
    GRCh37,
    GRCh38,
}
/// See [docs::HEADER_DOCS] and [docs::EXAMPLE_HEADER].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Study {
    /// Version of the scoring file format, e.g. '2.0'.
    format_version: String,
    pgs_info: PgsInfo,
    source_info: SourceInfo,
    associations: Vec<StudyAssociation>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgsInfo {
    /// PGS identifier, e.g. 'PGS000001'
    id: PgsId,
    /// PGS name, e.g. 'PRS77_BC' - optional
    name: Option<String>,
    /// trait, e.g. 'Breast Cancer'
    trait_reported: String,
    /// Ontology trait name, e.g. 'breast carcinoma'
    trait_mapped: String,
    /// Ontology trait ID (EFO), e.g. 'EFO_0000305'
    trait_efo: String,
    /// Genome build/assembly, e.g. 'GRCh38'
    genome_build: String,
    /// Number of variants listed in the PGS
    variant_number: String,
    /// Variant weight type, e.g. 'beta', 'OR/HR' (default 'NR')
    weight_type: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    /// PGS publication identifier, e.g. 'PGP000001'
    pgp_id: String,
    /// Information about the publication
    citation: String,
    /// License and terms of PGS use/distribution - refers to the EMBL-EBI Terms
    /// of Use by default
    license: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarmonizationInfo {
    /// Genome build of the harmonized file, e.g. 'GRCh38'>
    #[serde(rename = "HmPOS_build")]
    genome_build: String,
    /// Date of the harmonized file creation, e.g. '2022-05-26'>
    #[serde(rename = "HmPOS_date")]
    file_creation: String,
    // /// Number of entries matching and not matching the given chromosome,
    // /// e.g. {"True": 5210, "False": 8}>
    // #[serde(rename = "HmPOS_match_chr")]
    // HmPOS_match_chr: String,
    // /// Number of entries matching and not matching the given position,
    // /// e.g. {"True": 5210, "False": 8}>
    // #[serde(rename = "HmPOS_match_pos")]
    // HmPOS_match_pos: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyAssociation {
    /// The SNP’s rs ID.
    /// This column also contains HLA alleles in the standard notation (e.g.
    /// HLA-DQA1*0102) that aren’t always provided with chromosomal positions.
    /// See [boilerplate::invalid_rsid].
    #[serde(rename = "rsID")]
    // #[serde(deserialize_with = "invalid_rsid")]
    pub rs_id: Option<String>,
    /// Chromosome name/number associated with the variant.
    pub chr_name: Option<String>,
    /// Chromosomal position associated with the variant.
    pub chr_position: Option<u64>,
    /// The allele that's dosage is counted (e.g. {0, 1, 2}) and multiplied by
    /// the variant's weight (effect_weight) when calculating score. The effect
    /// allele is also known as the 'risk allele'. Note: this does not
    /// necessarily need to correspond to the minor allele/alternative allele.
    pub effect_allele: Allele,
    /// The other allele(s) at the loci. Note: this does not necessarily need to
    /// correspond to the reference allele.
    /// See [docs::OTHER_ALLELE] for what kind of values are present in [Allele::Other].
    pub other_allele: Option<Allele>,
    /// This is kept in for loci where the variant may be referenced by the gene
    /// (APOE e4). It is also common (usually in smaller PGS) to see the
    /// variants named according to the genes they impact.
    pub locus_name: Option<String>,
    /// This is a TRUE/FALSE variable that flags whether the effect allele is a
    /// haplotype/diplotype rather than a single SNP. Constituent SNPs in the
    /// haplotype are semi-colon separated.
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_haplotype: Option<bool>,
    /// This is a TRUE/FALSE variable that flags whether the effect allele is a
    /// haplotype/diplotype rather than a single SNP. Constituent SNPs in the
    /// haplotype are semi-colon separated.
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_diplotype: Option<bool>,
    /// This described whether the variant was specifically called with a
    /// specific imputation or variant calling method. This is mostly kept to
    /// describe HLA-genotyping methods (e.g. flag SNP2HLA, HLA*IMP) that gives
    /// alleles that are not referenced by genomic position.
    pub imputation_method: Option<ImputationMethod>,
    /// This field describes any extra information about the variant (e.g. how
    /// it is genotyped or scored) that cannot be captured by the other fields.
    pub variant_description: Option<String>,
    /// Explanation of when this variant gets included into the PGS (e.g. if it
    /// depends on the results from other variants).
    pub inclusion_criteria: Option<String>,

    /// Value of the effect that is multiplied by the dosage of the effect
    /// allele (effect_allele) when calculating the score. Additional
    /// information on how the effect_weight was derived is in the weight_type
    /// field of the header, and score development method in the metadata
    /// downloads.
    pub effect_weight: Option<NotNan<f64>>,
    /// This is a TRUE/FALSE variable that flags whether the weight should be
    /// multiplied with the dosage of more than one variant. Interactions are
    /// demarcated with a _x_ between entries for each of the variants present
    /// in the interaction.
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_interaction: Option<bool>,
    /// This is a TRUE/FALSE variable that flags whether the weight should be
    /// added to the PGS sum if there is at least 1 copy of the effect allele
    /// (e.g. it is a dominant allele).
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_dominant: Option<bool>,
    /// This is a TRUE/FALSE variable that flags whether the weight should be
    /// added to the PGS sum only if there are 2 copies of the effect allele
    /// (e.g. it is a recessive allele).
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_recessive: Option<bool>,
    /// Weights that are specific to different dosages of the effect_allele
    /// (e.g. {0, 1, 2} copies) can also be reported when the the contribution
    /// of the variants to the score is not encoded as additive, dominant, or
    /// recessive. In this case three columns are added corresponding to which
    /// variant weight should be applied for each dosage, where the column name
    /// is formated as dosage_#_weight where the # sign indicates the number of
    /// effect_allele copies.
    pub dosage_0_weight: Option<NotNan<f64>>,
    /// See [StudyAssociation::dosage_0_weight].
    pub dosage_1_weight: Option<NotNan<f64>>,
    /// See [StudyAssociation::dosage_0_weight].
    pub dosage_2_weight: Option<NotNan<f64>>,

    /// Author-reported effect sizes can be supplied to the Catalog. If no other
    /// effect_weight is given the weight is calculated using the log(OR) or
    /// log(HR).
    #[serde(rename = "OR")]
    pub or: Option<NotNan<f64>>,
    /// Author-reported effect sizes can be supplied to the Catalog. If no other
    /// effect_weight is given the weight is calculated using the log(OR) or
    /// log(HR).
    #[serde(rename = "HR")]
    pub hr: Option<NotNan<f64>>,

    /// Reported effect allele frequency, if the associated locus is a haplotype
    /// then haplotype frequency will be extracted.
    pub allelefrequency_effect: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_European")]
    pub allelefrequency_effect_european: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_Asian")]
    pub allelefrequency_effect_asian: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_African")]
    pub allelefrequency_effect_african: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_Hispanic")]
    pub allelefrequency_effect_hispanic: Option<NotNan<f64>>,

    pub variant_type: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarmonizedStudyAssociation {
    /// The SNP’s rs ID.
    /// This column also contains HLA alleles in the standard notation (e.g.
    /// HLA-DQA1*0102) that aren’t always provided with chromosomal positions.
    /// See [boilerplate::invalid_rsid].
    #[serde(rename = "rsID")]
    // #[serde(deserialize_with = "invalid_rsid")]
    pub rs_id: Option<String>,
    /// Chromosome name/number associated with the variant.
    pub chr_name: Option<String>,
    /// Chromosomal position associated with the variant.
    pub chr_position: Option<u64>,
    /// The allele that's dosage is counted (e.g. {0, 1, 2}) and multiplied by
    /// the variant's weight (effect_weight) when calculating score. The effect
    /// allele is also known as the 'risk allele'. Note: this does not
    /// necessarily need to correspond to the minor allele/alternative allele.
    pub effect_allele: Allele,
    /// The other allele(s) at the loci. Note: this does not necessarily need to
    /// correspond to the reference allele.
    /// See [docs::OTHER_ALLELE] for what kind of values are present in [Allele::Other].
    pub other_allele: Option<Allele>,
    /// This is kept in for loci where the variant may be referenced by the gene
    /// (APOE e4). It is also common (usually in smaller PGS) to see the
    /// variants named according to the genes they impact.
    pub locus_name: Option<String>,
    /// This is a TRUE/FALSE variable that flags whether the effect allele is a
    /// haplotype/diplotype rather than a single SNP. Constituent SNPs in the
    /// haplotype are semi-colon separated.
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_haplotype: Option<bool>,
    /// This is a TRUE/FALSE variable that flags whether the effect allele is a
    /// haplotype/diplotype rather than a single SNP. Constituent SNPs in the
    /// haplotype are semi-colon separated.
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_diplotype: Option<bool>,
    /// This described whether the variant was specifically called with a
    /// specific imputation or variant calling method. This is mostly kept to
    /// describe HLA-genotyping methods (e.g. flag SNP2HLA, HLA*IMP) that gives
    /// alleles that are not referenced by genomic position.
    pub imputation_method: Option<ImputationMethod>,
    /// This field describes any extra information about the variant (e.g. how
    /// it is genotyped or scored) that cannot be captured by the other fields.
    pub variant_description: Option<String>,
    /// Explanation of when this variant gets included into the PGS (e.g. if it
    /// depends on the results from other variants).
    pub inclusion_criteria: Option<String>,

    /// Value of the effect that is multiplied by the dosage of the effect
    /// allele (effect_allele) when calculating the score. Additional
    /// information on how the effect_weight was derived is in the weight_type
    /// field of the header, and score development method in the metadata
    /// downloads.
    pub effect_weight: Option<NotNan<f64>>,
    /// This is a TRUE/FALSE variable that flags whether the weight should be
    /// multiplied with the dosage of more than one variant. Interactions are
    /// demarcated with a _x_ between entries for each of the variants present
    /// in the interaction.
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_interaction: Option<bool>,
    /// This is a TRUE/FALSE variable that flags whether the weight should be
    /// added to the PGS sum if there is at least 1 copy of the effect allele
    /// (e.g. it is a dominant allele).
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_dominant: Option<bool>,
    /// This is a TRUE/FALSE variable that flags whether the weight should be
    /// added to the PGS sum only if there are 2 copies of the effect allele
    /// (e.g. it is a recessive allele).
    #[serde(default)]
    #[serde(with = "boilerplate::pgs_bool")]
    pub is_recessive: Option<bool>,
    /// Weights that are specific to different dosages of the effect_allele
    /// (e.g. {0, 1, 2} copies) can also be reported when the the contribution
    /// of the variants to the score is not encoded as additive, dominant, or
    /// recessive. In this case three columns are added corresponding to which
    /// variant weight should be applied for each dosage, where the column name
    /// is formated as dosage_#_weight where the # sign indicates the number of
    /// effect_allele copies.
    pub dosage_0_weight: Option<NotNan<f64>>,
    /// See [StudyAssociation::dosage_0_weight].
    pub dosage_1_weight: Option<NotNan<f64>>,
    /// See [StudyAssociation::dosage_0_weight].
    pub dosage_2_weight: Option<NotNan<f64>>,

    /// Author-reported effect sizes can be supplied to the Catalog. If no other
    /// effect_weight is given the weight is calculated using the log(OR) or
    /// log(HR).
    #[serde(rename = "OR")]
    pub or: Option<NotNan<f64>>,
    /// Author-reported effect sizes can be supplied to the Catalog. If no other
    /// effect_weight is given the weight is calculated using the log(OR) or
    /// log(HR).
    #[serde(rename = "HR")]
    pub hr: Option<NotNan<f64>>,

    /// Reported effect allele frequency, if the associated locus is a haplotype
    /// then haplotype frequency will be extracted.
    pub allelefrequency_effect: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_European")]
    pub allelefrequency_effect_european: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_Asian")]
    pub allelefrequency_effect_asian: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_African")]
    pub allelefrequency_effect_african: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    #[serde(rename = "allelefrequency_effect_Hispanic")]
    pub allelefrequency_effect_hispanic: Option<NotNan<f64>>,

    pub variant_type: Option<String>,

    // -------------------------------------------
    // Harmonized-only fields start here
    // -------------------------------------------
    /// Provider of the harmonized variant information
    ///
    /// Data source of the variant position. Options include: ENSEMBL, liftover,
    /// author-reported (if being harmonized to the same build).
    #[serde(rename = "hm_source")]
    pub source: HarmonizedSource,
    /// Harmonized rsID
    ///
    /// Current rsID. Differences between this column and the author-reported
    /// column (rsID) indicate variant merges and annotation updates from dbSNP.
    #[serde(rename = "hm_rsID")]
    pub hm_rs_id: Option<RsId>,
    /// Harmonized chromosome name
    ///
    /// Chromosome that the harmonized variant is present on, preferring matches
    /// to chromosomes over patches present in later builds.
    #[serde(rename = "hm_chr")]
    pub chr: String,
    /// Harmonized chromosome position
    ///
    /// Chromosomal position (base pair location) where the variant is located,
    /// preferring matches to chromosomes over patches present in later builds.
    #[serde(rename = "hm_pos")]
    pub pos: Option<u64>,
    /// Harmonized other alleles
    ///
    /// If only the effect_allele is given we attempt to infer the
    /// non-effect/other allele(s) using Ensembl/dbSNP alleles.
    /// See [docs::OTHER_ALLELE_INFERRED] for what kind of values are present in [Allele::Other].
    #[serde(rename = "hm_inferOtherAllele")]
    pub infer_other_allele: Option<Allele>,
    /// FLAG: matching chromosome name
    ///
    /// Used for QC. Only provided if the scoring file is being harmonized to
    /// the same genome build, and where the chromosome name is provided in the
    /// column chr_name.
    #[serde(default)]
    #[serde(rename = "hm_match_chr")]
    #[serde(with = "boilerplate::pgs_bool")]
    pub match_chr: Option<bool>,
    /// FLAG: matching chromosome position
    ///
    /// Used for QC. Only provided if the scoring file is being harmonized to
    /// the same genome build, and where the chromosome name is provided in the
    /// column chr_position.
    #[serde(default)]
    #[serde(rename = "hm_match_pos")]
    #[serde(with = "boilerplate::pgs_bool")]
    pub match_pos: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Allele {
    Insertion,
    Sequence(DnaSequence),
    /// See [docs::DOCUMENTED_EXCEPTIONS] for exceptions.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImputationMethod {
    #[serde(rename = "HLA*IMP:02")]
    HLAIMP02,
    #[serde(rename = "SNP2HLA")]
    SNP2HLA,
    #[serde(rename = "TopMed")]
    TopMed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarmonizedSource {
    #[serde(rename = "Author-reported")]
    AuthorReported,
    #[serde(rename = "ENSEMBL")]
    Ensembl,
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "liftover")]
    Liftover,
}

impl Study {
    pub async fn load(
        id: PgsId,
    ) -> Result<impl Iterator<Item = Result<StudyAssociation, csv::Error>>, std::io::Error> {
        let fs_entry = download_and_cache_study(id).await?;
        let mut file = BufReader::new(MultiGzDecoder::new(fs_entry.get()?));
        let _header = comments::read(&mut file)?;
        Ok(read_file(file))
    }

    fn url(id: PgsId) -> Url {
        format!("https://ftp.ebi.ac.uk/pub/databases/spot/pgs/scores/{id}/ScoringFiles/{id}.txt.gz")
            .parse()
            .unwrap()
    }
    fn path(id: PgsId) -> PathBuf {
        format!("scores/{id}/ScoringFiles/{id}.txt.gz")
            .parse()
            .unwrap()
    }
}

impl HarmonizedStudy {
    pub async fn load(
        id: PgsId,
        build: GenomeBuild,
    ) -> Result<impl Iterator<Item = Result<HarmonizedStudyAssociation, csv::Error>>, std::io::Error>
    {
        let fs_entry = download_and_cache_harmonized_study(id, build).await?;
        let mut file = BufReader::new(MultiGzDecoder::new(fs_entry.get()?));
        let _header = comments::read(&mut file)?;
        Ok(read_file(file))
    }

    fn url(id: PgsId, build: GenomeBuild) -> Url {
        format!("https://ftp.ebi.ac.uk/pub/databases/spot/pgs/scores/{id}/ScoringFiles/Harmonized/{id}_hmPOS_{build}.txt.gz")
            .parse()
            .unwrap()
    }
    fn path(id: PgsId, build: GenomeBuild) -> PathBuf {
        format!("scores/{id}/ScoringFiles/Harmonized/{id}_hmPOS_{build}.txt.gz")
            .parse()
            .unwrap()
    }
}

async fn download_and_cache_study(id: PgsId) -> Result<CacheEntry, std::io::Error> {
    let url: Url = Study::url(id);
    let path: PathBuf = Study::path(id);

    let fs_entry = Cache::global("pgs_catalog").entry(path);

    UrlEntry::new(url)
        .unwrap()
        .get_and_cache_async(&format!("[Data][PGS Catalog][{id}]"), fs_entry.clone())
        .await?;

    Ok(fs_entry)
}

async fn download_and_cache_harmonized_study(
    id: PgsId,
    build: GenomeBuild,
) -> Result<CacheEntry, std::io::Error> {
    let url: Url = HarmonizedStudy::url(id, build);
    let path: PathBuf = HarmonizedStudy::path(id, build);

    let fs_entry = Cache::global("pgs_catalog").entry(path);

    UrlEntry::new(url)
        .unwrap()
        .get_and_cache_async(
            &format!("[Data][PGS Catalog][{id}][Harmonized {build}]"),
            fs_entry.clone(),
        )
        .await?;

    Ok(fs_entry)
}

fn read_file<T: DeserializeOwned>(file: impl Read) -> impl Iterator<Item = Result<T, csv::Error>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .trim(csv::Trim::Fields)
        .from_reader(file)
        .into_deserialize()
}

pub mod comments {
    use std::io;

    pub fn read(reader: &mut impl io::BufRead) -> Result<String, std::io::Error> {
        let mut vec = vec![];

        while let [b'#', ..] = reader.fill_buf()? {
            reader.read_until(b'\n', &mut vec)?;
        }

        String::from_utf8(vec).map_err(utile::io::invalid_data)
    }
    #[allow(dead_code)]
    pub fn skip(reader: &mut impl io::BufRead) -> Result<(), std::io::Error> {
        while let [b'#', ..] = reader.fill_buf()? {
            reader.skip_until(b'\n')?;
        }
        Ok(())
    }

    #[test]
    fn test_skip_comments() -> io::Result<()> {
        use std::io::{Cursor, Read};

        let mut reader = Cursor::new(
            "##Comment 1\n##Comment 2\n##Comment 3\n#Header\nMore content\n##Ignored comment",
        );
        skip(&mut reader)?;

        let mut v = String::new();
        reader.read_to_string(&mut v)?;
        assert_eq!(v, "#Header\nMore content\n##Ignored comment");

        Ok(())
    }
}

#[allow(dead_code)]
mod docs {
    pub(super) const HEADER_DOCS: &str = "
###PGS CATALOG SCORING FILE - see https://www.pgscatalog.org/downloads/#dl_ftp_scoring for additional information
#format_version=<Version of the scoring file format, e.g. '2.0'>
##POLYGENIC SCORE (PGS) INFORMATION
#pgs_id=<PGS identifier, e.g. 'PGS000001'>
#pgs_name=<PGS name, e.g. 'PRS77_BC' - optional>
#trait_reported=<trait, e.g. 'Breast Cancer'>
#trait_mapped=<Ontology trait name, e.g. 'breast carcinoma'>
#trait_efo=<Ontology trait ID (EFO), e.g. 'EFO_0000305'>
#genome_build=<Genome build/assembly, e.g. 'GRCh38'>
#variants_number=<Number of variants listed in the PGS>
#weight_type=<Variant weight type, e.g. 'beta', 'OR/HR' (default 'NR')>
##SOURCE INFORMATION
#pgp_id=<PGS publication identifier, e.g. 'PGP000001'>
#citation=<Information about the publication>
#license=<License and terms of PGS use/distribution - refers to the EMBL-EBI Terms of Use by default>
rsID	chr_name	chr_position	effect_allele	other_allele	effect_weight	...
";

    pub(super)const EXAMPLE_HEADER: &str = "
###PGS CATALOG SCORING FILE - see https://www.pgscatalog.org/downloads/#dl_ftp_scoring for additional information
#format_version=2.0
##POLYGENIC SCORE (PGS) INFORMATION
#pgs_id=PGS000079
#pgs_name=CC_Melanoma
#trait_reported=Melanoma
#trait_mapped=melanoma
#trait_efo=EFO_0000756
#weight_type=log(OR)
#genome_build=GRCh37
#variants_number=24
##SOURCE INFORMATION
#pgp_id=PGP000050
#citation=Graff RE et al. bioRxiv (2020). doi:10.1101/2020.01.18.911578
rsID	chr_name	chr_position	effect_allele	other_allele	effect_weight	allelefrequency_effect	variant_description	OR
";

    pub(super) const HARMONIZATION_EXTENSION: &str = r###"
##HARMONIZATION DETAILS
#HmPOS_build=<Genome build of the harmonized file, e.g. 'GRCh38'>
#HmPOS_date=<Date of the harmonized file creation, e.g. '2022-05-26'>
#HmPOS_match_chr=<Number of entries matching and not matching the given chromosome, e.g. {"True": 5210, "False": 8}>
#HmPOS_match_pos=<Number of entries matching and not matching the given position, e.g. {"True": 5210, "False": 8}>
rsID	...	hm_source	hm_rsID	hm_chr	hm_pos	hm_inferOtherAllele	hm_match_chr	hm_match_pos
"###;

    pub(super) const OTHER_ALLELE: &[&str] = &[
        "ATGATTGTAAGTT+4",
        "ATTATTGTTTATG+4",
        "TTCTGCATGCATA+12",
        "<CN0>",
        "CC,AAA",
        "-",
        "CTAAATAACACAT+3",
        ".",
        "CCAGTACTTGAGG+38",
        "GT,GTT,GT",
        "N",
        "P",
        "CA,A",
        "CGATATTTGTCCC+4",
        "ATCTATCTATCTA+8",
        "TG,A",
        "GTT,AT",
        "T/C",
        "ATGAATTTAGCAT+17",
        "GATCCTTTTTAAA+6",
        "CTT,TTT",
    ];
    pub(super) const OTHER_ALLELE_INFERRED: &[&str] = &[
        "Y", "A/T", "-", "C/G/N/T", "K", "A/G", "G/T", "C/T", "C/G/T", "A/G/T", "A/C/G", "C/G",
        "A/C", "A/C/T",
    ];

    pub(super) const DOCUMENTED_EXCEPTIONS: &[&str] = &[
        "-", 
        "?", 
        "*01:01", 
        "*03:01; *03:04", 
        "*04:01; *04:03; *04:04; *04:05; *04:06;*04:07; *04:08; *04:10", 
        "*07:01", 
        "*08:01; *08:02; *08:03", 
        "*09:01", 
        "*10:01", 
        "*11:01; *11:03; *11:06; *11:08; *13:01; *13:02;*14:01; *14:02; *14:05; *14:06; *14:07; *14:03;*14:44", 
        "*12:01; *12:02; *12:05", 
        "*12", 
        "*15:01; *15:02; *16:02", 
        "*4", 
        "+", 
        "<CN0>", 
        "<CN2>", 
        "<CN3>", 
        "<INS:ME:ALU>", 
        "<INS:ME:LINE1>", 
        "<INS:ME:SVA>", 
        "<INS:MT>", 
        "<INV>", 
        "A24", 
        "B*08:01", 
        "B*27:05", 
        "B*39:01", 
        "B*44:02", 
        "C*06:02", 
        "CYP2A6*1", 
        "CYP2A6*12", 
        "CYP2A6*17", 
        "CYP2A6*1x2", 
        "CYP2A6*20", 
        "CYP2A6*25", 
        "CYP2A6*26", 
        "CYP2A6*27", 
        "CYP2A6*35", 
        "CYP2A6*4", 
        "CYP2A6*9", 
        "D10", 
        "D9", 
        "DQ2.2/DQ2.2", 
        "DQ2.2/DQ8", 
        "DQ2.2/X", 
        "DQ2.5/DQ2.2", 
        "DQ2.5/DQ2.5", 
        "DQ2.5/DQ8", 
        "DQ2.5/X", 
        "DQ7.5/DQ2.2", 
        "DQ7.5/DQ2.5", 
        "DQ7.5/DQ7.5", 
        "DQ7.5/DQ8", 
        "DQ7.5/X", 
        "DQ8/DQ8", 
        "DQ8/X", 
        "DR3/3", 
        "DR3/4", 
        "DR3/DR3", 
        "DR3/DR4-DQ8", 
        "DR3/X", 
        "DR4- DQ8/DR4-DQ8", 
        "DR4-DQ8/X", 
        "DR4/4", 
        "DR4/X", 
        "DRX/X", 
        "e2;e2", 
        "e2;e3", 
        "e2;e4", 
        "e2", 
        "e2/e2", 
        "e2/e3", 
        "e2/e4", 
        "e3;e2", 
        "e3;e3", 
        "e3;e4", 
        "e3/e3", 
        "e3/e4", 
        "e4;e2", 
        "e4;e3", 
        "e4;e4", 
        "e4", 
        "e4/e4", 
        "GCTCTGGCTCTCT+8", 
        "GTGATTGTAAGTT+4", 
        "GTTATTGTTTATG+4", 
        "HLA_A_77_30018741_N", 
        "HLA_A_95_30019036_L", 
        "HLA_A_95_30019036_V", 
        "HLA_A*03:01", 
        "HLA_B_08_01", 
        "HLA_B_18_01", 
        "HLA_B_45_31432581_E", 
        "HLA_B_67_31432515_C", 
        "HLA_B_67_31432515_F", 
        "HLA_B_67_31432515_M", 
        "HLA_B_67_31432515_Y", 
        "HLA_B_9_31432689_D", 
        "HLA_B_9_31432689_H", 
        "HLA_B_pos_116_Ser", 
        "HLA_B*08:01", 
        "HLA_B*18:01", 
        "HLA_B*27:02", 
        "HLA_B*27:05", 
        "HLA_C_04_01", 
        "HLA_C*06:02", 
        "HLA_C*12:03", 
        "HLA_DPB1_9_33156439_F", 
        "HLA_DPB1_pos_36_Ala", 
        "HLA_DQA1_01_02", 
        "HLA_DQA1_01_04", 
        "HLA_DQA1_04_01", 
        "HLA_DQA1_53_32717209_Q", 
        "HLA_DQA1_53_32717209_R", 
        "HLA_DQA1*01:02", 
        "HLA_DQB1_03_02", 
        "HLA_DQB1_pos_30_Tyr", 
        "HLA_DQB1*02:01", 
        "HLA_DQB1*03:02", 
        "HLA_DRB1_01_02", 
        "HLA_DRB1_03_01", 
        "HLA_DRB1_08_03", 
        "HLA_DRB1_11_13_71_74_I_", 
        "HLA_DRB1_11_13_71_74_II", 
        "HLA_DRB1_11_13_71_74_III", 
        "HLA_DRB1_11_13_71_74_IV", 
        "HLA_DRB1_11_13_71_74_IX", 
        "HLA_DRB1_11_13_71_74_I�", 
        "HLA_DRB1_11_13_71_74_V", 
        "HLA_DRB1_11_13_71_74_VI", 
        "HLA_DRB1_11_13_71_74_VII", 
        "HLA_DRB1_11_13_71_74_VIII", 
        "HLA_DRB1_11_13_71_74_XI", 
        "HLA_DRB1_11_13_71_74_XII", 
        "HLA_DRB1_11_13_71_74_XIII", 
        "HLA_DRB1_11_13_71_74_XIV", 
        "HLA_DRB1_11_13_71_74_XIX", 
        "HLA_DRB1_11_13_71_74_XV", 
        "HLA_DRB1_11_13_71_74_XVI", 
        "HLA_DRB1_11_13_71_74_XVII", 
        "HLA_DRB1_11_13_71_74_XVIII", 
        "HLA_DRB1_11_32660115_S/ *_L", 
        "HLA_DRB1_12_01", 
        "HLA_DRB1_pos_12_Lys", 
        "HLA_DRB1_pos_67_Ile", 
        "HLA_DRB1_pos_86_Val", 
        "HLA_DRB1*03:01", 
        "HLA-A*02:01", 
        "HLA-B*38:01", 
        "HLA-B*44:02", 
        "HLA-DQA1*01:01", 
        "HLA-DQA1*01:02;HLA-DQB1*06:02_x_HLA-DQA1*01:02;HLA-DQB1*06:02", 
        "HLA-DQA1*01:02;HLA-DQB1*06:02", 
        "HLA-DQA1*01:02;HLA-DQB1*06:09", 
        "HLA-DQA1*01:03;HLA-DQB1*06:01", 
        "HLA-DQA1*01:03;HLA-DQB1*06:03", 
        "HLA-DQA1*01:0X;HLA-DQB1*05:01", 
        "HLA-DQA1*01:0X;HLA-DQB1*05:03", 
        "HLA-DQA1*02:01;HLA-DQB1*02:02", 
        "HLA-DQA1*02:01;HLA-DQB1*03:03", 
        "HLA-DQA1*03:02;HLA-DQB1*03:03_x_HLA-DQA1*03:0X;HLA-DQB1*03:02", 
        "HLA-DQA1*03:02;HLA-DQB1*03:03", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:01_x_HLA-DQA1*01:0X;HLA-DQB1*05:01", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:01_x_HLA-DQA1*02:01;HLA-DQB1*02:02", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:01_x_HLA-DQA1*03:0X;HLA-DQB1*03:01", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:01_x_HLA-DQA1*03:0X;HLA-DQB1*03:02", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:01_x_HLA-DQA1*05:05;HLA-DQB1*03:01", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:01", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:02_x_HLA-DQA1*01:03;HLA-DQB1*06:03", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:02_x_HLA-DQA1*03:0X;HLA-DQB1*03:02", 
        "HLA-DQA1*03:0X;HLA-DQB1*03:02", 
        "HLA-DQA1*04:01;HLA-DQB1*04:02_x_HLA-DQA1*02:01;HLA-DQB1*02:02", 
        "HLA-DQA1*04:01;HLA-DQB1*04:02", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*01:02;HLA-DQB1*06:02", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*01:03;HLA-DQB1*06:03", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*01:0X;HLA-DQB1*05:01", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*02:01;HLA-DQB1*02:02", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*03:0X;HLA-DQB1*03:01", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*03:0X;HLA-DQB1*03:02", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*05:01;HLA-DQB1*02:01", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01_x_HLA-DQA1*05:05;HLA-DQB1*03:01", 
        "HLA-DQA1*05:01;HLA-DQB1*02:01", 
        "HLA-DQA1*05:05;HLA-DQB1*03:01", 
        "HLA-DRB1*03:01", 
        "HLA-DRB1*08:01", 
        "HLA-DRB1*13:03", 
        "HLA-DRB1*15:01", 
        "K", 
        "N",
        "P", 
        "R", 
        "S", 
        "TATCCTTTTTAAA+6", 
        "TCAGTACTTGAGG+38", 
        "TGATATTTGTCCC+4", 
        "TGGCA,GTGG,CAGC", 
        "TTAAATAACACAT+3", 
        "V", 
        "X", 
        "X/X", 
        "Y", 
    ];
}

mod boilerplate {
    use std::fmt;

    use ids::rs::RsId;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use biocore::{
        dna::{DnaBase, DnaSequence},
        sequence::AsciiChar,
    };

    use crate::GenomeBuild;

    use super::Allele;

    impl fmt::Display for GenomeBuild {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                GenomeBuild::GRCh37 => f.write_str("GRCh37"),
                GenomeBuild::GRCh38 => f.write_str("GRCh38"),
            }
        }
    }

    impl Serialize for Allele {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                Allele::Insertion => "I".serialize(serializer),
                Allele::Sequence(sequence) => sequence.serialize(serializer),
                Allele::Other(other) => other.serialize(serializer),
            }
        }
    }
    impl<'de> Deserialize<'de> for Allele {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            match &*s {
                "I" => Ok(Allele::Insertion),
                "D" => Ok(Allele::Sequence(DnaSequence::new(vec![]))),
                _ => {
                    if let Ok(s) = DnaBase::decode(s.clone().into_bytes()) {
                        Ok(Allele::Sequence(s))
                    } else {
                        Ok(Allele::Other(s))
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn invalid_rsid<'de, D>(de: D) -> Result<Option<RsId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        const OTHER_VALUES: &[&str] = &[
            "exm657395",
            "exm2259631",
            "exm1356144",
            "exm2273005",
            "rs2187668,rs7454108",
            "rs9273369_x_rs9275490",
            "rs1281935_x_rs10947332",
            "rs9275490_x_rs9275490",
        ];
        let s = String::deserialize(de)?;
        match s.parse() {
            Ok(rsid) => Ok(Some(rsid)),
            Err(_) if s.is_empty() => Ok(None),
            Err(_) if OTHER_VALUES.contains(&&*s) => todo!(),
            Err(_) => todo!(),
        }
    }

    pub mod pgs_bool {
        use std::fmt;

        use serde::Serialize;

        pub fn serialize<S>(v: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match *v {
                Some(true) => "True".serialize(serializer),
                Some(false) => "False".serialize(serializer),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Option<bool>;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a boolean value (e.g. 'True'/'TRUE' or 'False'/'FALSE')")
                }
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match v{
                        "True" | "TRUE" => Ok(Some(true)),
                        "False" |"FALSE" => Ok(Some(false)),
                        "" => unreachable!(),
                        _ => Err(serde::de::Error::custom(format!(
                            "invalid boolean value. Expected 'True'/'TRUE' or 'False'/'FALSE', found '{v}'."
                        ))),
                    }
                }
                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(v)
                }
                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(&v)
                }
                fn visit_none<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(None)
                }
                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(None)
                }
                fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str(self)
                }

                fn __private_visit_untagged_option<D>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, ()>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str(self).map_err(drop)
                }
            }
            deserializer.deserialize_option(Visitor)
        }
    }
}
