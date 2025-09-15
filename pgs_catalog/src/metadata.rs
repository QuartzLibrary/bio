use std::{
    io::Read,
    path::{Path, PathBuf},
};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tar::Archive;

use ids::{
    pgs::{PgsId, pgp::PgpId, ppm::PpmId, pss::PssId},
    pubmed::PubmedId,
};
use url::Url;
use utile::resource::{RawResource, RawResourceExt};

use crate::{PgsCatalogResource, WeightType};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Metadata {
    pub cohorts: Vec<Cohort>,
    pub evaluation_sample_sets: Vec<EvaluationSampleSet>,
    pub performance_metrics: Vec<PerformanceMetric>,
    pub score_development_samples: Vec<ScoreDevelopmentSample>,
    pub scores: Vec<Score>,
    pub efo_traits: Vec<EfoTrait>,
    pub publications: Vec<Publication>,
}
impl Metadata {
    pub async fn load_all() -> Result<Self, std::io::Error> {
        load_all_metadata(None).await
    }
    pub async fn load(id: PgsId) -> Result<Self, std::io::Error> {
        load_all_metadata(Some(id)).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cohort {
    #[serde(rename = "Cohort ID")]
    pub id: String,
    #[serde(rename = "Cohort Name")]
    pub name: String,
    #[serde(rename = "Previous/other/additional names")]
    pub other_names: Option<String>,
}
impl Cohort {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_cohorts.csv").parse().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationSampleSet {
    #[serde(rename = "PGS Sample Set (PSS)")]
    pub pgs_sample_set: PssId,
    #[serde(rename = "Polygenic Score (PGS) ID")]
    #[serde(with = "SpaceCommaSeparated")]
    pub score_ids: Vec<PgsId>,
    #[serde(rename = "Number of Individuals")]
    pub number_of_individuals: usize,
    #[serde(rename = "Number of Cases")]
    #[serde(with = "decimal_usize")]
    pub number_of_cases: Option<usize>,
    #[serde(rename = "Number of Controls")]
    #[serde(with = "decimal_usize")]
    pub number_of_controls: Option<usize>,
    #[serde(rename = "Percent of Participants Who are Male")]
    pub percent_of_participants_who_are_male: Option<NotNan<f64>>,
    #[serde(rename = "Sample Age")]
    pub sample_age: String,
    #[serde(rename = "Broad Ancestry Category")]
    pub broad_ancestry_category: String,
    #[serde(rename = "Ancestry (e.g. French, Chinese)")]
    pub ancestry: String,
    #[serde(rename = "Country of Recruitment")]
    pub country_of_recruitment: String,
    #[serde(rename = "Additional Ancestry Description")]
    pub additional_ancestry_description: String,
    #[serde(rename = "Phenotype Definitions and Methods")]
    pub phenotype_definitions_and_methods: String,
    #[serde(rename = "Followup Time")]
    pub followup_time: String,
    #[serde(rename = "GWAS Catalog Study ID (GCST...)")]
    pub gwas_catalog_study_id: String,
    #[serde(rename = "Source PubMed ID (PMID)")]
    #[serde(with = "decimal_pmid")]
    pub source_pubmed_id: Option<PubmedId>,
    #[serde(rename = "Source DOI")]
    pub source_doi: String,
    #[serde(rename = "Cohort(s)")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub cohorts: Vec<String>,
    #[serde(rename = "Additional Sample/Cohort Information")]
    pub additional_sample_cohort_information: String,
}
impl EvaluationSampleSet {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_evaluation_sample_sets.csv")
            .parse()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceMetric {
    #[serde(rename = "PGS Performance Metric (PPM) ID")]
    pub id: PpmId,
    #[serde(rename = "Evaluated Score")]
    pub evaluated_score: PgsId,
    #[serde(rename = "PGS Sample Set (PSS)")]
    pub pgs_sample_set: PssId,
    #[serde(rename = "PGS Publication (PGP) ID")]
    pub pgs_publication_id: PgpId,
    #[serde(rename = "Reported Trait")]
    pub reported_trait: String,
    #[serde(rename = "Covariates Included in the Model")]
    pub covariates_included_in_the_model: String,
    #[serde(rename = "PGS Performance: Other Relevant Information")]
    pub other_relevant_information: String,
    #[serde(rename = "Publication (PMID)")]
    pub publication_pmid: Option<PubmedId>,
    #[serde(rename = "Publication (doi)")]
    pub publication_doi: String,
    #[serde(rename = "Hazard Ratio (HR)")]
    pub hazard_ratio: String,
    #[serde(rename = "Odds Ratio (OR)")]
    pub odds_ratio: String,
    #[serde(rename = "Beta")]
    pub beta: String,
    #[serde(rename = "Area Under the Receiver-Operating Characteristic Curve (AUROC)")]
    pub auroc: String,
    #[serde(rename = "Concordance Statistic (C-index)")]
    pub concordance_statistic_c_index: String,
    #[serde(rename = "Other Metric(s)")]
    pub other_metrics: String,
}
impl PerformanceMetric {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_performance_metrics.csv")
            .parse()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoreDevelopmentSample {
    #[serde(rename = "Polygenic Score (PGS) ID")]
    pub score_id: PgsId,
    #[serde(rename = "Stage of PGS Development")]
    pub stage_of_pgs_development: StageOfPgsDevelopment,
    #[serde(rename = "Number of Individuals")]
    #[serde(with = "decimal_usize")]
    pub number_of_individuals: Option<usize>,
    #[serde(rename = "Number of Cases")]
    #[serde(with = "decimal_usize")]
    pub number_of_cases: Option<usize>,
    #[serde(rename = "Number of Controls")]
    #[serde(with = "decimal_usize")]
    pub number_of_controls: Option<usize>,
    #[serde(rename = "Percent of Participants Who are Male")]
    pub percent_of_participants_who_are_male: Option<NotNan<f64>>,
    #[serde(rename = "Sample Age")]
    pub sample_age: String,
    #[serde(rename = "Broad Ancestry Category")]
    pub broad_ancestry_category: String,
    #[serde(rename = "Ancestry (e.g. French, Chinese)")]
    pub ancestry: String,
    #[serde(rename = "Country of Recruitment")]
    pub country_of_recruitment: String,
    #[serde(rename = "Additional Ancestry Description")]
    pub additional_ancestry_description: String,
    #[serde(rename = "Phenotype Definitions and Methods")]
    pub phenotype_definitions_and_methods: String,
    #[serde(rename = "Followup Time")]
    pub followup_time: String,
    #[serde(rename = "GWAS Catalog Study ID (GCST...)")]
    pub gwas_catalog_study_id: String,
    #[serde(rename = "Source PubMed ID (PMID)")]
    pub source_pubmed_id: Option<PubmedId>,
    #[serde(rename = "Source DOI")]
    pub source_doi: String,
    #[serde(rename = "Cohort(s)")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub cohorts: Vec<String>,
    #[serde(rename = "Additional Sample/Cohort Information")]
    pub additional_sample_cohort_information: String,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum StageOfPgsDevelopment {
    #[serde(rename = "Score Development/Training")]
    ScoreDevelopmentTraining,
    #[serde(rename = "Source of Variant Associations (GWAS)")]
    SourceOfVariantAssociationsGwas,
}
impl ScoreDevelopmentSample {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_score_development_samples.csv")
            .parse()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Score {
    #[serde(rename = "Polygenic Score (PGS) ID")]
    pub id: PgsId,
    #[serde(rename = "PGS Name")]
    pub name: String,
    #[serde(rename = "Reported Trait")]
    pub reported_trait: String,
    #[serde(rename = "Mapped Trait(s) (EFO label)")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub mapped_traits_efo_label: Vec<String>,
    #[serde(rename = "Mapped Trait(s) (EFO ID)")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub mapped_traits_efo_id: Vec<String>,
    #[serde(rename = "PGS Development Method")]
    pub pgs_development_method: String,
    #[serde(rename = "PGS Development Details/Relevant Parameters")]
    pub pgs_development_details_and_relevant_parameters: String,
    #[serde(rename = "Original Genome Build")]
    pub original_genome_build: String,
    #[serde(rename = "Number of Variants")]
    pub number_of_variants: usize,
    #[serde(rename = "Number of Interaction Terms")]
    pub number_of_interaction_terms: usize,
    #[serde(rename = "Type of Variant Weight")]
    pub type_of_variant_weight: WeightType,
    #[serde(rename = "PGS Publication (PGP) ID")]
    pub pgs_publication_id: PgpId,
    #[serde(rename = "Publication (PMID)")]
    pub publication_pmid: Option<PubmedId>,
    #[serde(rename = "Publication (doi)")]
    pub publication_doi: String,
    #[serde(rename = "Score and results match the original publication")]
    #[serde(with = "pgs_bool")]
    pub score_and_results_match_the_original_publication: bool,
    #[serde(rename = "Ancestry Distribution (%) - Source of Variant Associations (GWAS)")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub ancestry_distribution_source_of_variant_associations_gwas: Vec<String>,
    #[serde(rename = "Ancestry Distribution (%) - Score Development/Training")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub ancestry_distribution_score_development_and_training: Vec<String>,
    #[serde(rename = "Ancestry Distribution (%) - PGS Evaluation")]
    #[serde(with = "utile::serde_ext::PipeSeparated")]
    pub ancestry_distribution_pgs_evaluation: Vec<String>,
    #[serde(rename = "FTP link")]
    pub ftp_link: Url,
    #[serde(rename = "Release Date")]
    pub release_date: String,
    #[serde(rename = "License/Terms of Use")]
    pub license_and_terms_of_use: String,
}
impl Score {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_scores.csv").parse().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EfoTrait {
    #[serde(rename = "Ontology Trait ID")]
    pub id: String,
    #[serde(rename = "Ontology Trait Label")]
    pub label: String,
    #[serde(rename = "Ontology Trait Description")]
    pub description: String,
    #[serde(rename = "Ontology URL")]
    pub url: Url,
}
impl EfoTrait {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_efo_traits.csv").parse().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    #[serde(rename = "PGS Publication/Study (PGP) ID")]
    pub id: PgpId,
    #[serde(rename = "First Author")]
    pub first_author: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Journal Name")]
    pub journal_name: String,
    #[serde(rename = "Publication Date")]
    pub publication_date: String,
    #[serde(rename = "Release Date")]
    pub release_date: String,
    #[serde(rename = "Authors")]
    pub authors: String,
    #[serde(rename = "digital object identifier (doi)")]
    pub doi: String,
    #[serde(rename = "PubMed ID (PMID)")]
    pub pmid: Option<PubmedId>,
}
impl Publication {
    pub async fn load_all() -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(None, Self::file_name(None)).await
    }
    pub async fn load(id: PgsId) -> Result<Vec<Self>, std::io::Error> {
        load_metadata_file(Some(id), Self::file_name(Some(id))).await
    }
    fn file_name(id: Option<PgsId>) -> PathBuf {
        let prefix = prefix(id);
        format!("{prefix}_metadata_publications.csv")
            .parse()
            .unwrap()
    }
}

async fn load_all_metadata(id: Option<PgsId>) -> Result<Metadata, std::io::Error> {
    let cohorts_path = Cohort::file_name(id);
    let evaluation_sample_sets_path = EvaluationSampleSet::file_name(id);
    let performance_metrics_path = PerformanceMetric::file_name(id);
    let score_development_samples_path = ScoreDevelopmentSample::file_name(id);
    let scores_path = Score::file_name(id);
    let efo_traits_path = EfoTrait::file_name(id);
    let publications_path = Publication::file_name(id);

    let empty: PathBuf = "/".parse().unwrap();
    let xlsx: PathBuf = format!("{}_metadata.xlsx", prefix(id)).parse().unwrap();

    let mut cohorts: Option<Vec<Cohort>> = None;
    let mut evaluation_sample_sets: Option<Vec<EvaluationSampleSet>> = None;
    let mut performance_metrics: Option<Vec<PerformanceMetric>> = None;
    let mut score_development_samples: Option<Vec<ScoreDevelopmentSample>> = None;
    let mut scores: Option<Vec<Score>> = None;
    let mut efo_traits: Option<Vec<EfoTrait>> = None;
    let mut publications: Option<Vec<Publication>> = None;

    let resource = PgsCatalogResource::Metadata { id }
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?
        .decompressed();

    let mut archive = Archive::new(resource.read()?);

    for entry in archive.entries()? {
        let entry = entry?;
        let path = &*entry.path()?;
        if path == cohorts_path {
            cohorts = Some(read_file(entry)?);
        } else if path == evaluation_sample_sets_path {
            evaluation_sample_sets = Some(read_file(entry)?);
        } else if path == performance_metrics_path {
            performance_metrics = Some(read_file(entry)?);
        } else if path == score_development_samples_path {
            score_development_samples = Some(read_file(entry)?);
        } else if path == scores_path {
            scores = Some(read_file(entry)?);
        } else if path == efo_traits_path {
            efo_traits = Some(read_file(entry)?);
        } else if path == publications_path {
            publications = Some(read_file(entry)?);
        } else if path == empty || path == xlsx {
            continue;
        } else {
            log::warn!(
                "[Data][PGS Catalog] Foun an unexpected file in metadata bundle: {}",
                entry.path().unwrap().display()
            );
            continue;
        }
    }

    Ok(Metadata {
        cohorts: cohorts.ok_or(missing(cohorts_path))?,
        evaluation_sample_sets: evaluation_sample_sets
            .ok_or(missing(evaluation_sample_sets_path))?,
        performance_metrics: performance_metrics.ok_or(missing(performance_metrics_path))?,
        score_development_samples: score_development_samples
            .ok_or(missing(score_development_samples_path))?,
        scores: scores.ok_or(missing(scores_path))?,
        efo_traits: efo_traits.ok_or(missing(efo_traits_path))?,
        publications: publications.ok_or(missing(publications_path))?,
    })
}

async fn load_metadata_file<T: DeserializeOwned>(
    id: Option<PgsId>,
    file_name: impl AsRef<Path>,
) -> Result<Vec<T>, std::io::Error> {
    let file_name = file_name.as_ref();

    let resource = PgsCatalogResource::Metadata { id }
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?
        .decompressed();

    let mut archive = Archive::new(resource.read()?);

    for entry in archive.entries()? {
        let entry = entry?;
        if &*entry.path()? == file_name {
            return Ok(read_file(entry)?);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        missing(file_name),
    ))?
}
fn read_file<T: DeserializeOwned>(file: impl Read) -> Result<Vec<T>, csv::Error> {
    Ok(csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(file)
        .into_deserialize()
        .try_collect()
        .unwrap())
}

fn missing(path: impl AsRef<Path>) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!(
            "Missing file from metadata bundle: {:?}",
            path.as_ref().display()
        ),
    )
}

fn prefix(id: Option<PgsId>) -> String {
    match id {
        Some(id) => id.to_string(),
        None => "pgs_all".to_owned(),
    }
}

mod pgs_bool {
    use std::fmt;

    use serde::Serialize;

    pub fn serialize<S>(v: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match v {
            true => "True".serialize(serializer),
            false => "False".serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = bool;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a boolean value (e.g. 'True'/'TRUE' or 'False'/'FALSE')")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "True" | "TRUE" => Ok(true),
                    "False" | "FALSE" => Ok(false),
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
        }
        deserializer.deserialize_str(Visitor)
    }
}

mod decimal_pmid {
    use ids::pubmed::PubmedId;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(v: &Option<PubmedId>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        v.map(|v| format!("{}.0", v.inner()))
            .unwrap_or_default()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PubmedId>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.is_empty() {
            return Ok(None);
        }

        let id = s
            .strip_suffix(".0")
            .unwrap_or(&s)
            .parse::<u64>()
            .map_err(serde::de::Error::custom)?;

        Ok(Some(PubmedId::new(id)))
    }
}

mod decimal_usize {
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(v: &Option<usize>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        v.map(|v| format!("{v}.0"))
            .unwrap_or_default()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.is_empty() {
            return Ok(None);
        }

        let id = s
            .strip_suffix(".0")
            .unwrap_or(&s)
            .parse::<usize>()
            .map_err(serde::de::Error::custom)?;

        Ok(Some(id))
    }
}

struct SpaceCommaSeparator;
impl utile::serde_ext::Separator for SpaceCommaSeparator {
    fn separator() -> &'static str {
        ", "
    }
}
type SpaceCommaSeparated<T> = utile::serde_ext::Separated<SpaceCommaSeparator, T>;

mod boilerplate {
    use std::fmt::Display;

    use serde::Serialize;

    use super::StageOfPgsDevelopment;

    impl Display for StageOfPgsDevelopment {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", serialize(self))
        }
    }

    fn serialize<T: Serialize>(v: &T) -> String {
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.serialize(v).unwrap();
        let s = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        s.trim_matches('"').to_owned()
    }
}
