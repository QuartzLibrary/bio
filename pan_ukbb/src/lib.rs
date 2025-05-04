#![feature(iterator_try_collect)]

use std::{collections::BTreeSet, io, mem};

use biocore::{
    dna::DnaSequence,
    location::{GenomePosition, GenomeRange, SequenceOrientation},
};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use url::Url;

use utile::resource::{RawResource, RawResourceExt, UrlResource};

const URL_BASE: &str = "https://pan-ukb-us-east-1.s3.amazonaws.com";
const PHENOTYPE_MANIFEST_KEY: &str = "sumstats_release/phenotype_manifest.tsv.bgz";

pub struct PanUKBBS3Resource {
    pub key: String,
}
impl PanUKBBS3Resource {
    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn phenotype_manifest() -> Self {
        Self::new(PHENOTYPE_MANIFEST_KEY.to_owned())
    }

    pub fn url(&self) -> Url {
        let key = &self.key;
        Url::parse(&format!("{URL_BASE}/{key}")).unwrap()
    }
    fn url_resource(&self) -> UrlResource {
        UrlResource::new(self.url()).unwrap()
    }
}
impl RawResource for PanUKBBS3Resource {
    const NAMESPACE: &'static str = "pan_ukbb";

    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<utile::resource::Compression> {
        if self.key.ends_with(".gz") || self.key.ends_with(".bgz") {
            Some(utile::resource::Compression::MultiGzip)
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

/// https://pan.ukbb.broadinstitute.org/docs/per-phenotype-files
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(non_snake_case)]
pub struct PhenotypeManifestEntry {
    // Phenotype ID fields
    // The first 5 fields are guaranteed to be unique.
    /// One of the following: continuous, biomarkers, prescriptions, icd10, phecode, categorical
    pub trait_type: TraitType,
    /// The code for the phenotype (for continuous, biomarkers, and categorical traits,
    /// this corresponds to the field ID as described by UKB, e.g. 21001 for BMI)
    pub phenocode: String,
    pub pheno_sex: PhenoSex,
    /// For categorical variables, this corresponds to the coding that was used
    /// (e.g. coding 2 for field 1747 [hair color]). For all other trait_types, this field is blank.
    pub coding: Option<String>,
    /// Refers to any miscellaneous downstream modifications of the phenotype
    /// (e.g. irnt for inverse-rank normal transformation).
    /// If the phenotype is updated, this field can be used to denote the update
    /// (e.g. the particular wave of COVID-19 data used).
    pub modifier: Option<Modifier>,
    /// A shorter description of the phenotype (for continuous, biomarkers, and categorical variables,
    /// corresponds to the Description on the showcase). For phecodes, this is the "description"
    /// column in the phecodes definition file.
    pub description: String,
    /// A longer description of the phenotype (for continuous and categorical variables,
    /// corresponds to the Notes page on the showcase).
    pub description_more: String,
    /// For categorical variables, a description of the particular coding that was used
    /// (the Meaning column on the showcase page for that coding).
    pub coding_description: String,
    /// A categorization of the phenotype. For continuous, biomarkers, and categorical
    /// traits, this corresponds to the Category at the top of the showcase page.
    /// For ICD codes, this corresponds to the Chapter of the ICD code; for phecodes,
    /// this is the "group" column in the phecodes definition file; for prescriptions,
    /// this corresponds to a semi-manual categorization of prescription drugs.
    pub category: Option<String>,
    /// If the phenotype is in our maximally indepdent set. This set of relatively uncorrelated
    /// phenotypes was constructed using a pairwise phenotypic correlation matrix of phenotypes
    /// with ancestries passing all QC filters (released via make_pairwise_ht).
    /// Of all phenotype pairs, we retained any with a pairwise correlation r<0.1r<0.1.
    /// For pairs with r>0.1r>0.1 , we used hl.maximal_independent_set to identify indendent
    /// phenotypes for retention, imposing a tiebreaker of higher case count (or higher sample
    /// size for continuous phenotypes), producing 195 independent phenotypes.
    pub in_max_independent_set: bool,

    // Case and ancestry fields
    /// Number of cases (or individuals phenotyped for quantitative traits) across all ancestry groups,
    /// females and males combined. Should be similar to the sum of per-ancestry n_cases for relevant
    /// ancestries, but may include ancestry outliers and samples that failed QC.c
    pub n_cases_full_cohort_both_sexes: usize,
    /// Number of female cases (or individuals phenotyped for quantitative traits) across all ancestry
    /// groups. May include ancestry outliers and samples that failed QC.
    pub n_cases_full_cohort_females: usize,
    /// Number of male cases (or individuals phenotyped for quantitative traits) across all ancestry
    /// groups. May include ancestry outliers and samples that failed QC.
    pub n_cases_full_cohort_males: usize,
    /// Number of cases (or individuals phenotyped for quantitative traits) across ancestry groups passing
    /// stringent phenotype QC (see pops_pass_qc), females and males combined. Should be similar to the
    /// sum of per-ancestry n_cases for relevant ancestries, but may include ancestry outliers and samples
    /// that failed QC.
    #[serde(with = "s::opt")]
    pub n_cases_hq_cohort_both_sexes: Option<usize>,
    /// Number of female cases (or individuals phenotyped for quantitative traits) across ancestry groups
    /// passing stringent phenotype QC (see pops_pass_qc). May include ancestry outliers and samples that failed QC.
    #[serde(with = "s::opt")]
    pub n_cases_hq_cohort_females: Option<usize>,
    /// Number of male cases (or individuals phenotyped for quantitative traits) across ancestry groups
    /// passing stringent phenotype QC (see pops_pass_qc). May include ancestry outliers and samples that failed QC.
    #[serde(with = "s::opt")]
    pub n_cases_hq_cohort_males: Option<usize>,
    /// List of ancestry codes for which this phenotypes was GWASed.
    #[serde(with = "s::comma")]
    pub pops: BTreeSet<Population>,
    /// Number of ancestry groups for which this phenotype was GWASed
    pub num_pops: usize,
    /// Comma-delimited list of ancestry codes for which this phenotype passes QC (see quality control,
    /// heritability manifest, and phenotype_qc_{pop} field).
    #[serde(with = "s::comma")]
    pub pops_pass_qc: BTreeSet<Population>,
    /// Number of ancestry groups for which this phenotype passes QC.
    pub num_pops_pass_qc: usize,

    // Population-specific fields
    #[serde(with = "s::opt")]
    pub n_cases_AFR: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_cases_AMR: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_cases_CSA: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_cases_EAS: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_cases_EUR: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_cases_MID: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_controls_AFR: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_controls_AMR: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_controls_CSA: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_controls_EAS: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_controls_EUR: Option<usize>,
    #[serde(with = "s::opt")]
    pub n_controls_MID: Option<usize>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub sldsc_25bin_h2_observed_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_se_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_se_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_se_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_se_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub sldsc_25bin_h2_observed_se_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_observed_se_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub sldsc_25bin_h2_liability_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_se_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_se_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_se_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_se_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub sldsc_25bin_h2_liability_se_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_liability_se_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_z_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_z_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_z_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_z_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub sldsc_25bin_h2_z_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub rhemc_25bin_50rv_h2_z_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub lambda_gc_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub lambda_gc_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub lambda_gc_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub lambda_gc_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub lambda_gc_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub lambda_gc_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub phenotype_qc_AFR: Option<PhenotypeQc>,
    #[serde(with = "s::opt")]
    pub phenotype_qc_AMR: Option<PhenotypeQc>,
    #[serde(with = "s::opt")]
    pub phenotype_qc_CSA: Option<PhenotypeQc>,
    #[serde(with = "s::opt")]
    pub phenotype_qc_EAS: Option<PhenotypeQc>,
    #[serde(with = "s::opt")]
    pub phenotype_qc_EUR: Option<PhenotypeQc>,
    #[serde(with = "s::opt")]
    pub phenotype_qc_MID: Option<PhenotypeQc>,

    // File information
    /// Name of summary statistics file.
    pub filename: String,
    pub filename_tabix: String,
    /// Link to download summary statistics file from Amazon AWS.
    pub aws_path: Url,
    pub aws_path_tabix: Url,
    pub md5_hex: String,
    pub size_in_bytes: usize,
    pub md5_hex_tabix: String,
    pub size_in_bytes_tabix: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub enum TraitType {
    #[serde(rename = "biomarkers")]
    Biomarkers,
    #[serde(rename = "categorical")]
    Categorical,
    #[serde(rename = "continuous")]
    Continuous,
    #[serde(rename = "icd10")]
    ICD10,
    #[serde(rename = "phecode")]
    PheCode,
    #[serde(rename = "prescriptions")]
    Prescriptions,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub enum PhenoSex {
    #[serde(rename = "both_sexes")]
    Both,
    #[serde(rename = "females")]
    Female,
    #[serde(rename = "males")]
    Male,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub enum Modifier {
    #[serde(rename = "02")]
    Modifier02,
    #[serde(rename = "03")]
    Modifier03,
    #[serde(rename = "04")]
    Modifier04,
    #[serde(rename = "CPD_combined_irnt")]
    CpdCombinedIrnt,
    #[serde(rename = "CPD_combined_raw")]
    CpdCombinedRaw,
    #[serde(rename = "Ever_Never")]
    EverNever,
    #[serde(rename = "auto_irnt")]
    AutoIrnt,
    #[serde(rename = "auto_medadj_irnt")]
    AutoMedadjIrnt,
    #[serde(rename = "auto_medadj_raw")]
    AutoMedadjRaw,
    #[serde(rename = "auto_raw")]
    AutoRaw,
    #[serde(rename = "combined_irnt")]
    CombinedIrnt,
    #[serde(rename = "combined_medadj_irnt")]
    CombinedMedadjIrnt,
    #[serde(rename = "combined_medadj_raw")]
    CombinedMedadjRaw,
    #[serde(rename = "combined_raw")]
    CombinedRaw,
    #[serde(rename = "irnt")]
    Irnt,
    #[serde(rename = "manual_irnt")]
    ManualIrnt,
    #[serde(rename = "manual_medadj_irnt")]
    ManualMedadjIrnt,
    #[serde(rename = "manual_medadj_raw")]
    ManualMedadjRaw,
    #[serde(rename = "manual_raw")]
    ManualRaw,
    #[serde(rename = "medadj_irnt")]
    MedadjIrnt,
    #[serde(rename = "medadj_raw")]
    MedadjRaw,
    #[serde(rename = "random_strat")]
    RandomStrat,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Population {
    Afr,
    Amr,
    Csa,
    Eas,
    Eur,
    Mid,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub enum PhenotypeQc {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "fail_ratio")]
    FailRatio,
    #[serde(rename = "h2_not_defined")]
    H2NotDefined,
    #[serde(rename = "h2_z_insignificant")]
    H2ZInsignificant,
    #[serde(rename = "not_EUR_plus_1")]
    NotEurPlus1,
    #[serde(rename = "out_of_bounds_h2")]
    OutOfBoundsH2,
    #[serde(rename = "out_of_bounds_lambda")]
    OutOfBoundsLambda,
    #[serde(rename = "n_too_low")]
    NTooLow,
}

impl PhenotypeManifestEntry {
    pub async fn load_all() -> csv::Result<Vec<Self>> {
        let resource = PanUKBBS3Resource::phenotype_manifest()
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached_async()
            .await?
            .decompressed()
            .buffered();

        csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(resource.read()?)
            .into_deserialize()
            .try_collect()
    }
    pub fn passes_qc(&self) -> bool {
        // Kind of arbitrary, just to shave the number.
        self.pops_pass_qc.contains(&Population::Eur) && self.n_cases_full_cohort_both_sexes > 50_000
    }
}

impl PhenotypeManifestEntry {
    pub fn get_summary_stats_resource(&self) -> io::Result<PanUKBBS3Resource> {
        let key = format!("sumstats_flat_files/{}", self.filename);
        assert!(self.aws_path.as_str().ends_with(&key));
        Ok(PanUKBBS3Resource::new(key))
    }
    pub async fn get_summary_stats(
        &self,
    ) -> io::Result<impl Iterator<Item = csv::Result<SummaryStats>> + use<>> {
        let key = format!("sumstats_flat_files/{}", self.filename);
        assert!(self.aws_path.as_str().ends_with(&key));
        let resource = PanUKBBS3Resource::new(key.clone())
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached_async()
            .await?
            .decompressed()
            .buffered();

        Ok(csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(resource.read()?)
            .into_deserialize())
    }

    pub fn get_summary_stats_tabix_resource(&self) -> io::Result<PanUKBBS3Resource> {
        let key = format!("sumstats_release/{}", self.filename_tabix);
        assert!(self.aws_path_tabix.as_str().ends_with(&key));
        Ok(PanUKBBS3Resource::new(key))
    }
    pub async fn get_summary_stats_tabix(&self) -> io::Result<Vec<u8>> {
        let key = format!("sumstats_release/{}", self.filename_tabix);
        assert!(self.aws_path_tabix.as_str().ends_with(&key));
        PanUKBBS3Resource::new(key)
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached_async()
            .await?
            .read_vec_async()
            .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(non_snake_case)]
pub struct SummaryStats {
    // Variant fields
    /// Chromosome of the variant.
    pub chr: String,
    /// Position of the variant in GRCh37 coordinates.
    pub pos: u64,
    /// Reference allele on the forward strand.
    #[serde(rename = "ref")]
    pub ref_allele: DnaSequence,
    /// Alternate allele (not necessarily minor allele). Used as effect allele for GWAS.
    pub alt: DnaSequence,

    // High quality meta-analysis fields
    /// Alternate allele frequency from meta-analysis across populations for which this phenotype passes all QC filters.
    /// pub NOTE: This field only appears in files for quantitative phenotypes.
    #[serde(with = "s::opt", default)]
    pub af_meta_hq: Option<NotNan<f64>>,
    /// Estimated effect size of alternate allele from meta-analysis across populations for which this phenotype passes all QC filters.
    #[serde(with = "s::opt")]
    pub beta_meta_hq: Option<NotNan<f64>>,
    /// Estimated standard error of beta_meta_hq.
    #[serde(with = "s::opt")]
    pub se_meta_hq: Option<NotNan<f64>>,
    /// -log10 p-value of beta_meta_hq significance test.
    #[serde(with = "s::opt")]
    pub neglog10_pval_meta_hq: Option<NotNan<f64>>,
    /// -log10 p-value from heterogeneity test of meta-analysis.
    #[serde(with = "s::opt")]
    pub neglog10_pval_heterogeneity_hq: Option<NotNan<f64>>,

    // Meta-analysis fields
    /// Alternate allele frequency from meta-analysis across populations for which this phenotype was GWASed.
    /// pub NOTE: This field only appears in files for quantitative phenotypes.
    #[serde(with = "s::opt", default)]
    pub af_meta: Option<NotNan<f64>>,
    /// Estimated effect size of alternate allele from meta-analysis across populations for which this phenotype was GWASed.
    #[serde(with = "s::opt")]
    pub beta_meta: Option<NotNan<f64>>,
    /// Estimated standard error of beta_meta.
    #[serde(with = "s::opt")]
    pub se_meta: Option<NotNan<f64>>,
    /// -log10 p-value of beta_meta significance test.
    #[serde(with = "s::opt")]
    pub neglog10_pval_meta: Option<NotNan<f64>>,
    /// -log10 p-value from heterogeneity test of meta-analysis.
    #[serde(with = "s::opt")]
    pub neglog10_pval_heterogeneity: Option<NotNan<f64>>,

    // Population-specific fields
    #[serde(with = "s::opt")]
    pub af_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub af_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub af_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub af_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub af_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub af_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub beta_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub beta_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub beta_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub beta_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub beta_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub beta_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub se_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub se_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub se_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub se_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub se_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub se_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub neglog10_pval_AFR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub neglog10_pval_AMR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub neglog10_pval_CSA: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub neglog10_pval_EAS: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub neglog10_pval_EUR: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub neglog10_pval_MID: Option<NotNan<f64>>,
    #[serde(with = "s::opt")]
    pub low_confidence_AFR: Option<bool>,
    #[serde(with = "s::opt")]
    pub low_confidence_AMR: Option<bool>,
    #[serde(with = "s::opt")]
    pub low_confidence_CSA: Option<bool>,
    #[serde(with = "s::opt")]
    pub low_confidence_EAS: Option<bool>,
    #[serde(with = "s::opt")]
    pub low_confidence_EUR: Option<bool>,
    #[serde(with = "s::opt")]
    pub low_confidence_MID: Option<bool>,
}
impl SummaryStats {
    pub fn at(&self) -> GenomePosition {
        GenomePosition {
            name: self.chr.clone(),
            orientation: SequenceOrientation::Forward,
            at: self.pos - 1,
        }
    }
    pub fn at_range(&self) -> GenomeRange {
        GenomeRange {
            name: self.chr.clone(),
            orientation: SequenceOrientation::Forward,
            at: self.pos - 1..(self.pos - 1 + u64::try_from(self.ref_allele.len()).unwrap()),
        }
    }

    pub fn flip_ref_alt(&mut self) {
        mem::swap(&mut self.ref_allele, &mut self.alt);

        self.af_meta = None;
        self.af_meta_hq = None;
        self.af_AFR = None;
        self.af_AMR = None;
        self.af_CSA = None;
        self.af_EAS = None;
        self.af_EUR = None;
        self.af_MID = None;

        self.beta_meta = self.beta_meta.map(|b| -b);
        self.beta_meta_hq = self.beta_meta_hq.map(|b| -b);
        self.beta_AFR = self.beta_AFR.map(|b| -b);
        self.beta_AMR = self.beta_AMR.map(|b| -b);
        self.beta_CSA = self.beta_CSA.map(|b| -b);
        self.beta_EAS = self.beta_EAS.map(|b| -b);
        self.beta_EUR = self.beta_EUR.map(|b| -b);
        self.beta_MID = self.beta_MID.map(|b| -b);
    }
}

mod s {
    pub mod opt {
        use std::{fmt, marker::PhantomData};

        use serde::{
            Deserialize, Serialize,
            de::{
                DeserializeOwned,
                value::{F32Deserializer, F64Deserializer, U64Deserializer},
            },
        };

        use utile::serde_ext::StringDeserializer;

        pub fn serialize<S, T>(v: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
            T: Serialize,
        {
            match v {
                Some(v) => v.serialize(serializer),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
        where
            D: serde::Deserializer<'de>,
            T: DeserializeOwned,
        {
            struct Visitor<T>(PhantomData<T>);
            impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
            where
                T: DeserializeOwned,
            {
                type Value = Option<T>;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    let type_name = std::any::type_name::<T>();
                    write!(formatter, "either 'NA', '', or a value of type {type_name}")
                }
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match v {
                        "" | "NA" => Ok(None),
                        _ => Ok(Some(<T as Deserialize>::deserialize(
                            StringDeserializer::new(v),
                        )?)),
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
                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    <T as Deserialize>::deserialize(U64Deserializer::new(v)).map(Some)
                }
                fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    <T as Deserialize>::deserialize(F32Deserializer::new(v)).map(Some)
                }
                fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    <T as Deserialize>::deserialize(F64Deserializer::new(v)).map(Some)
                }
                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(None)
                }
                fn visit_none<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(None)
                }
                fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_any(Visitor(PhantomData))
                }
                fn __private_visit_untagged_option<D>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, ()>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer
                        .deserialize_any(Visitor(PhantomData))
                        .map_err(drop)
                }
            }
            deserializer.deserialize_option(Visitor(PhantomData))
        }
    }
    pub mod comma {
        use std::{fmt, marker::PhantomData};

        use serde::{Deserialize, Serialize, de::DeserializeOwned};

        use utile::serde_ext::StringSequenceDeserializer;

        pub fn serialize<S, T>(_v: &T, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
            T: Serialize,
        {
            todo!()
        }

        pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
        where
            D: serde::Deserializer<'de>,
            T: DeserializeOwned,
        {
            struct Visitor<T>(PhantomData<T>);
            impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
            where
                T: DeserializeOwned,
            {
                type Value = T;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    let type_name = std::any::type_name::<T>();
                    write!(formatter, "either 'NA', '', or a value of type {type_name}")
                }
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    <T as Deserialize>::deserialize(StringSequenceDeserializer::new(v, ','))
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
            }
            deserializer.deserialize_str(Visitor(PhantomData))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[tokio::test]
    async fn test_get_latest_manifest() {
        let result = PhenotypeManifestEntry::load_all().await.unwrap();
        assert!(!result.is_empty(), "Manifest should not be empty");

        println!("Number of phenotypes: {}", result.len());

        // let count = result
        //     .iter()
        //     .filter(|e| e.n_cases_full_cohort_both_sexes)
        //     .count();
        // println!("count: {count:?}");

        let values: BTreeSet<_> = result.iter().map(|e| e.in_max_independent_set).collect();
        println!("{values:?}");
        println!("{:?}", values.iter().next());

        let values: usize = result.iter().map(|e| e.size_in_bytes).sum();
        println!("{values:?}");

        // // Log the first few entries for debugging
        // println!(
        //     "First entries: {:#?}",
        //     &result[..std::cmp::min(3, result.len())]
        // );
    }

    #[tokio::test]
    async fn test_get_summary_stats() {
        let manifest = PhenotypeManifestEntry::load_all().await.unwrap();
        let entry = manifest.first().unwrap();

        // Only fetch the first few rows to avoid a large download during tests
        // In a real scenario, you might want to process the stats differently
        let stats_result = entry.get_summary_stats().await.unwrap();

        // let mut count = 0;
        // let mut count_beta_meta_hq = 0;
        // for (i, e) in stats_result.enumerate() {
        //     if i % 100_000 == 0 {
        //         println!("{}", i);
        //     }
        //     let e = e.unwrap();
        //     count += 1;
        //     if e.beta_meta_hq.is_some() {
        //         count_beta_meta_hq += 1;
        //     }
        // }

        // println!("count: {count:?}");
        // println!("count_beta_meta_hq: {count_beta_meta_hq:?}");

        let values: BTreeSet<_> = stats_result.map(|e| e.unwrap().chr).collect();
        println!("{values:?}");
        // println!("{:?}", values.iter().next());

        // println!("{:?} {:?}", stats_result.len(), stats_result.first());
    }
}
