use either::Either;
use ids::pubmed::PubmedId;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};
use url::Url;

use biocore::location::ContigPosition;
use utile::{
    io::reqwest_error,
    resource::{RawResource, RawResourceExt, UrlResource},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GwasCatalogResource {
    url: &'static str,
    key: String,
    size: u64,
}
impl GwasCatalogResource {
    pub async fn get_latest_associations() -> std::io::Result<Self> {
        get_latest_key(Self::ASSOCIATIONS_URL).await
    }
    pub async fn get_latest_studies() -> std::io::Result<Self> {
        get_latest_key(Self::STUDIES_URL).await
    }
    pub async fn get_latest_ancestries() -> std::io::Result<Self> {
        get_latest_key(Self::ANCESTRY_URL).await
    }

    const ASSOCIATIONS_URL: &str = "https://www.ebi.ac.uk/gwas/api/search/downloads/alternative";
    const STUDIES_URL: &str = "https://www.ebi.ac.uk/gwas/api/search/downloads/studies/v1.0.3.1";
    const ANCESTRY_URL: &str =
        "https://www.ebi.ac.uk/gwas/api/search/downloads/ancestries/v1.0.3.1";
    pub fn associations_url(&self) -> Url {
        Self::ASSOCIATIONS_URL.parse().unwrap()
    }
    pub fn studies_url(&self) -> Url {
        Self::STUDIES_URL.parse().unwrap()
    }
    pub fn ancestry_url(&self) -> Url {
        Self::ANCESTRY_URL.parse().unwrap()
    }
}
impl RawResource for GwasCatalogResource {
    const NAMESPACE: &'static str = "gwas_catalog";

    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<utile::resource::Compression> {
        None
    }

    type Reader = <UrlResource as RawResource>::Reader;
    fn size(&self) -> std::io::Result<u64> {
        Ok(self.size)
    }

    fn read(&self) -> std::io::Result<Self::Reader> {
        UrlResource::new(self.url).unwrap().read()
    }

    type AsyncReader = <UrlResource as RawResource>::AsyncReader;
    async fn size_async(&self) -> std::io::Result<u64> {
        Ok(self.size)
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        UrlResource::new(self.url).unwrap().read_async().await
    }
}

/// *Available in associations download files
/// +Available in studies download files
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GwasCatalogAssociation {
    //v1.0
    /// DATE ADDED TO CATALOG* +: Date a study is published in the catalog
    #[serde(rename = "DATE ADDED TO CATALOG")]
    pub date_added_to_catalog: Date,
    /// PUBMEDID* +: PubMed identification number
    #[serde(rename = "PUBMEDID")]
    pub pubmedid: PubmedId,
    /// FIRST AUTHOR* +: Last name and initials of first author
    #[serde(rename = "FIRST AUTHOR")]
    pub first_author: String,
    /// DATE* +: Publication date (online (epub) date if available)
    #[serde(rename = "DATE")]
    pub date: Date,
    /// JOURNAL* +: Abbreviated journal name
    #[serde(rename = "JOURNAL")]
    pub journal: String,
    /// LINK* +: PubMed URL
    #[serde(rename = "LINK")]
    pub link: String,
    /// STUDY* +: Title of paper
    #[serde(rename = "STUDY")]
    pub study: String,
    /// DISEASE/TRAIT* +: Disease or trait examined in study
    #[serde(rename = "DISEASE/TRAIT")]
    pub disease_or_trait: String,
    /// INITIAL SAMPLE DESCRIPTION* +: Sample size and ancestry description for stage 1 of GWAS (summing across multiple Stage 1 populations, if applicable)
    #[serde(rename = "INITIAL SAMPLE SIZE")]
    pub initial_sample_size: String,
    /// REPLICATION SAMPLE DESCRIPTION* +: Sample size and ancestry description for subsequent replication(s) (summing across multiple populations, if applicable)
    #[serde(rename = "REPLICATION SAMPLE SIZE")]
    pub replication_sample_size: String,
    /// REGION*: Cytogenetic region associated with rs number
    #[serde(rename = "REGION")]
    pub region: String,
    /// CHR_ID*: Chromosome number associated with rs number
    #[serde(rename = "CHR_ID")]
    pub chr_id: String,
    /// CHR_POS*: Chromosomal position associated with rs number
    #[serde(rename = "CHR_POS")]
    pub chr_pos: String,
    /// REPORTED GENE(S)*: Gene(s) reported by author
    #[serde(rename = "REPORTED GENE(S)")]
    pub reported_genes: String,
    /// MAPPED GENE(S)*: Gene(s) mapped to the strongest SNP. If the SNP is located within a gene, that gene is listed. If the SNP is located within multiple genes, these genes are listed separated by commas. If the SNP is intergenic, the upstream and downstream genes are listed, separated by a hyphen.
    #[serde(rename = "MAPPED_GENE")]
    pub mapped_gene: String,
    /// UPSTREAM_GENE_ID*: Entrez Gene ID for nearest upstream gene to rs number, if not within gene
    #[serde(rename = "UPSTREAM_GENE_ID")]
    pub upstream_gene_id: String,
    /// DOWNSTREAM_GENE_ID*: Entrez Gene ID for nearest downstream gene to rs number, if not within gene
    #[serde(rename = "DOWNSTREAM_GENE_ID")]
    pub downstream_gene_id: String,
    /// SNP_GENE_IDS*: Entrez Gene ID, if rs number within gene; multiple genes denotes overlapping transcripts
    #[serde(rename = "SNP_GENE_IDS")]
    pub snp_gene_id: String,
    /// UPSTREAM_GENE_DISTANCE*: distance in kb for nearest upstream gene to rs number, if not within gene
    #[serde(rename = "UPSTREAM_GENE_DISTANCE")]
    pub upstream_gene_distance: String,
    /// DOWNSTREAM_GENE_DISTANCE*: distance in kb for nearest downstream gene to rs number, if not within gene
    #[serde(rename = "DOWNSTREAM_GENE_DISTANCE")]
    pub downstream_gene_distance: String,
    /// STRONGEST SNP-RISK ALLELE*: SNP(s) most strongly associated with trait + risk allele (? for unknown risk allele). May also refer to a haplotype.
    #[serde(rename = "STRONGEST SNP-RISK ALLELE")]
    pub strongest_snp_risk_allele: String,
    /// SNPS*: Strongest SNP; if a haplotype it may include more than one rs number (multiple SNPs comprising the haplotype)
    #[serde(rename = "SNPS")]
    pub snps: String,
    /// MERGED*: denotes whether the SNP has been merged into a subsequent rs record (0 = no; 1 = yes;)
    #[serde(rename = "MERGED")]
    pub merged: String,
    /// SNP_ID_CURRENT*: current rs number (will differ from strongest SNP when merged = 1)
    #[serde(rename = "SNP_ID_CURRENT")]
    pub snp_id_current: String,
    /// CONTEXT*: provides information on a variant’s predicted most severe functional effect from Ensembl
    #[serde(rename = "CONTEXT")]
    pub context: String,
    /// INTERGENIC*: denotes whether SNP is in intergenic region (0 = no; 1 = yes)
    #[serde(rename = "INTERGENIC")]
    pub intergenic: String,
    /// RISK ALLELE FREQUENCY*: Reported risk/effect allele frequency associated with strongest SNP in controls (if not available among all controls, among the control group with the largest sample size). If the associated locus is a haplotype the haplotype frequency will be extracted.
    #[serde(rename = "RISK ALLELE FREQUENCY")]
    pub risk_allele_frequency: String,
    /// P-VALUE*: Reported p-value for strongest SNP risk allele (linked to dbGaP Association Browser). Note that p-values are rounded to 1 significant digit (for example, a published p-value of 4.8 x 10-7 is rounded to 5 x 10-7).
    #[serde(rename = "P-VALUE")]
    pub p_value: String,
    /// PVALUE_MLOG*: -log(p-value)
    #[serde(rename = "PVALUE_MLOG")]
    pub p_value_mlog: String,
    /// P-VALUE (TEXT)*: Information describing context of p-value (e.g. females, smokers).
    #[serde(rename = "P-VALUE (TEXT)")]
    pub p_value_text: String,
    /// OR or BETA*: Reported odds ratio or beta-coefficient associated with strongest SNP risk allele. Note that prior to 2021, any OR <1 was inverted, along with the reported allele, so that all ORs included in the Catalog were >1. This is no longer done, meaning that associations added after 2021 may have OR <1. Appropriate unit and increase/decrease are included for beta coefficients.
    #[serde(rename = "OR or BETA")]
    pub or_or_beta: String,
    /// 95% CI (TEXT)*: Reported 95% confidence interval associated with strongest SNP risk allele, along with unit in the case of beta-coefficients. If 95% CIs are not published, we estimate these using the standard error, where available.
    #[serde(rename = "95% CI (TEXT)")]
    pub confidence_interval: String,
    /// PLATFORM (SNPS PASSING QC)*: Genotyping platform manufacturer used in Stage 1; also includes notation of pooled DNA study design or imputation of SNPs, where applicable
    #[serde(rename = "PLATFORM [SNPS PASSING QC]")]
    pub platform: String,
    /// CNV*: Study of copy number variation (yes/no)
    #[serde(rename = "CNV")]
    pub cnv: String,

    //v1.0.1
    /// MAPPED_TRAIT* +: Mapped Experimental Factor Ontology trait for this study
    #[serde(rename = "MAPPED_TRAIT")]
    pub mapped_trait: String,
    /// MAPPED_TRAIT_URI* +: URI of the EFO trait
    #[serde(rename = "MAPPED_TRAIT_URI")]
    pub mapped_trait_uri: Option<Url>,
    /// STUDY ACCESSION* +: Accession ID allocated to a GWAS Catalog study
    #[serde(rename = "STUDY ACCESSION")]
    pub study_accession: String,

    // v1.0.2
    /// GENOTYPING_TECHNOLOGY* +: Genotyping technology/ies used in this study, with additional array information (ex. Immunochip or Exome array) in brackets.
    #[serde(rename = "GENOTYPING TECHNOLOGY")]
    pub genotyping_technology: String,
    // v1.0.2.1 (none)
}
impl GwasCatalogAssociation {
    pub async fn get_latest()
    -> Result<impl Iterator<Item = Result<Self, csv::Error>>, std::io::Error> {
        let resource = GwasCatalogResource::get_latest_associations()
            .await?
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached_async()
            .await?
            .buffered();

        Ok(csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(resource.read()?)
            .into_deserialize())
    }
    pub fn locations_raw(&self) -> Vec<Location> {
        match self.locations() {
            Either::Left(locs) => locs,
            Either::Right(Interaction { a, b }) => vec![a, b],
        }
    }
    pub fn locations(&self) -> Either<Vec<Location>, Interaction> {
        // Assumes forward orientation, (not really specified in the docs).
        if self.chr_id.is_empty() {
            assert!(self.chr_pos.is_empty());
            assert!(self.region.is_empty());
            Either::Left(vec![])
        } else if self.chr_id.contains('x') {
            let (region_a, region_b) = self.region.split_once('x').unwrap();
            let (chr_id_a, chr_id_b) = self.chr_id.split_once('x').unwrap();
            let (chr_pos_a, chr_pos_b) = self.chr_pos.split_once('x').unwrap();

            Either::Right(Interaction {
                a: Location {
                    loc: ContigPosition {
                        contig: parse_human_contig(chr_id_a).unwrap().as_str().to_owned(),
                        at: chr_pos_a.trim().parse().unwrap(),
                    },
                    region: region_a.trim().to_owned(),
                },
                b: Location {
                    loc: ContigPosition {
                        contig: parse_human_contig(chr_id_b).unwrap().as_str().to_owned(),
                        at: chr_pos_b.trim().parse().unwrap(),
                    },
                    region: region_b.trim().to_owned(),
                },
            })
        } else {
            let mut region = self.region.split(';');
            let mut chr_id = self.chr_id.split(';');
            let mut chr_pos = self.chr_pos.split(';');

            let mut locations: Vec<Location> = vec![];

            let mut single_region = false;
            for i in 0.. {
                match (chr_id.next(), chr_pos.next()) {
                    (Some(chr_id), Some(chr_pos)) => {
                        let region = match region.next() {
                            Some(region) => region.to_owned(),
                            None if i == 1 || single_region => {
                                single_region = true;
                                locations[0].region.clone()
                            }
                            None => panic!("{self:?}"),
                        };
                        locations.push(Location {
                            loc: ContigPosition {
                                contig: parse_human_contig(chr_id).unwrap().as_str().to_owned(),
                                at: chr_pos.trim().parse().unwrap(),
                            },
                            region,
                        })
                    }
                    (None, None) => break,
                    _ => panic!("{self:?}"),
                }
            }

            Either::Left(locations)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GwasCatalogStudy {
    //v1.0
    /// DATE ADDED TO CATALOG* +: Date a study is published in the catalog
    #[serde(rename = "DATE ADDED TO CATALOG")]
    pub date_added_to_catalog: Date,
    /// PUBMEDID* +: PubMed identification number
    #[serde(rename = "PUBMED ID")]
    pub pubmedid: PubmedId,
    /// FIRST AUTHOR* +: Last name and initials of first author
    #[serde(rename = "FIRST AUTHOR")]
    pub first_author: String,
    /// DATE* +: Publication date (online (epub) date if available)
    #[serde(rename = "DATE")]
    pub date: Date,
    /// JOURNAL* +: Abbreviated journal name
    #[serde(rename = "JOURNAL")]
    pub journal: String,
    /// LINK* +: PubMed URL
    #[serde(rename = "LINK")]
    pub link: String,
    /// STUDY* +: Title of paper
    #[serde(rename = "STUDY")]
    pub study: String,
    /// DISEASE/TRAIT* +: Disease or trait examined in study
    #[serde(rename = "DISEASE/TRAIT")]
    pub disease_or_trait: String,
    /// INITIAL SAMPLE DESCRIPTION* +: Sample size and ancestry description for stage 1 of GWAS (summing across multiple Stage 1 populations, if applicable)
    #[serde(rename = "INITIAL SAMPLE SIZE")]
    pub initial_sample_size: String,
    /// REPLICATION SAMPLE DESCRIPTION* +: Sample size and ancestry description for subsequent replication(s) (summing across multiple populations, if applicable)
    #[serde(rename = "REPLICATION SAMPLE SIZE")]
    pub replication_sample_size: String,
    /// PLATFORM [SNPS PASSING QC] Genotyping platform manufacturer and number of SNPs tested in the analysis; also includes imputation of SNPs, where applicable
    #[serde(rename = "PLATFORM [SNPS PASSING QC]")]
    platform_snps_passing_qc: String,
    /// ASSOCIATION COUNT+: Number of associations identified for this study
    #[serde(rename = "ASSOCIATION COUNT")]
    pub association_count: u64,

    // v1.0.1
    /// MAPPED_TRAIT* +: Mapped Experimental Factor Ontology trait for this study
    #[serde(rename = "MAPPED_TRAIT")]
    pub mapped_trait: String,
    /// MAPPED_TRAIT_URI* +: URI of the EFO trait
    #[serde(rename = "MAPPED_TRAIT_URI")]
    pub mapped_trait_uri: String,
    /// STUDY ACCESSION* +: Accession ID allocated to a GWAS Catalog study
    #[serde(rename = "STUDY ACCESSION")]
    pub study_accession: String,

    // v1.0.2
    /// GENOTYPING_TECHNOLOGY* +: Genotyping technology/ies used in this study, with additional array information (ex. Immunochip or Exome array) in brackets.
    #[serde(rename = "GENOTYPING TECHNOLOGY")]
    pub genotyping_technology: String,

    /// SUBMISSION DATE The date the GWAS was submitted to the Catalog
    #[serde(rename = "SUBMISSION DATE")]
    submission_date: Option<Date>,
    /// STATISTICAL MODEL Details of the statistical model used to determine association significance
    #[serde(rename = "STATISTICAL MODEL")]
    statistical_model: String,
    /// BACKGROUND TRAIT Any background trait(s) shared by all individuals in the GWAS
    #[serde(rename = "BACKGROUND TRAIT")]
    background_trait: String,
    /// Undocumented
    #[serde(rename = "MAPPED BACKGROUND TRAIT")]
    mapped_background_trait: String,
    /// Undocumented
    #[serde(rename = "MAPPED BACKGROUND TRAIT URI")]
    mapped_background_trait_uri: String,

    // v1.0.2.1
    /// COHORT+: Discovery stage cohorts used in this study. The full list of cohort abbreviations and definitions can be found here.
    #[serde(rename = "COHORT")]
    pub cohort: String,
    /// FULL SUMMARY STATISTICS+: Availability of full genome-wide summary statistics files for download
    #[serde(rename = "FULL SUMMARY STATISTICS")]
    pub full_summary_statistics: String,
    /// SUMMARY STATS LOCATION+: The location of the summary statistics file
    #[serde(rename = "SUMMARY STATS LOCATION")]
    pub summary_stats_location: String,
}
impl GwasCatalogStudy {
    pub async fn get_latest()
    -> Result<impl Iterator<Item = Result<Self, csv::Error>>, std::io::Error> {
        let resource = GwasCatalogResource::get_latest_studies()
            .await?
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached_async()
            .await?
            .buffered();

        Ok(csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(resource.read()?)
            .into_deserialize())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GwasCatalogAncestry {
    // v1.0
    /// STUDY ACCESSION: Accession ID allocated to a GWAS Catalog study
    #[serde(rename = "STUDY ACCESSION")]
    pub study_accession: String,
    /// PUBMEDID: PubMed identification number
    #[serde(rename = "PUBMED ID")]
    pub pubmedid: PubmedId,
    /// FIRST AUTHOR: Last name and initials of first author
    #[serde(rename = "FIRST AUTHOR")]
    pub first_author: String,
    /// DATE: Publication date (online (epub) date if available)
    #[serde(rename = "DATE")]
    pub date: String,
    /// INITIAL SAMPLE DESCRIPTION: Sample size and ancestry description for GWAS stage (summing across multiple populations, if applicable)
    #[serde(rename = "INITIAL SAMPLE DESCRIPTION")]
    pub initial_sample_description: String,
    /// REPLICATION SAMPLE DESCRIPTION: Sample size and ancestry description for subsequent replication(s) (summing across multiple populations, if applicable)
    #[serde(rename = "REPLICATION SAMPLE DESCRIPTION")]
    pub replication_sample_description: String,
    /// STAGE: Stage of the GWAS to which the sample description applies, either initial or replication
    #[serde(rename = "STAGE")]
    pub stage: String,
    /// NUMBER OF INDIVDUALS: Number of individuals in this sample
    #[serde(rename = "NUMBER OF INDIVIDUALS")]
    pub number_of_indivduals: String,
    /// BROAD ANCESTRAL CATEGORY: Broad ancestral category to which the individuals in the sample belong
    #[serde(rename = "BROAD ANCESTRAL CATEGORY")]
    pub broad_ancestral_category: String,
    /// COUNTRY OF ORIGIN: Country of origin of the individuals in the sample
    #[serde(rename = "COUNTRY OF ORIGIN")]
    pub country_of_origin: String,
    /// COUNTRY OF RECRUITMENT: Country of recruitment of the individuals in the sample
    #[serde(rename = "COUNTRY OF RECRUITMENT")]
    pub country_of_recruitment: String,
    /// ADDITONAL ANCESTRY DESCRIPTION: Any additional ancestry descriptors relevant to the sample description
    #[serde(rename = "ADDITIONAL ANCESTRY DESCRIPTION")]
    pub additional_ancestry_description: String,

    // v1.0.3.1
    /// ANCESTRY DESCRIPTOR The most detailed ancestry descriptor(s) for the sample.
    #[serde(rename = "ANCESTRY DESCRIPTOR")]
    pub ancestry_descriptor: String,
    /// FOUNDER/GENETICALLY ISOLATED POPULATION Description of a founder or genetically isolated population
    #[serde(rename = "FOUNDER/GENETICALLY ISOLATED POPULATION")]
    pub founder_genetically_isolated_population: String,
    /// NUMBER OF CASES The number of cases in this broad ancestry group
    #[serde(rename = "NUMBER OF CASES")]
    pub number_of_cases: String,
    /// NUMBER OF CONTROLS The number of controls in this broad ancestry group
    #[serde(rename = "NUMBER OF CONTROLS")]
    pub number_of_controls: String,
    /// SAMPLE DESCRIPTION Additional sample information required for the interpretation of result
    #[serde(rename = "SAMPLE DESCRIPTION")]
    pub sample_description: String,
}
impl GwasCatalogAncestry {
    pub async fn get_latest()
    -> Result<impl Iterator<Item = Result<Self, csv::Error>>, std::io::Error> {
        let resource = GwasCatalogResource::get_latest_ancestries()
            .await?
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached_async()
            .await?
            .buffered();

        Ok(csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(resource.read()?)
            .into_deserialize())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interaction {
    pub a: Location,
    pub b: Location,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub loc: ContigPosition,
    pub region: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HumanContig {
    /// Chromosome 1
    Chr1,
    /// Chromosome 2
    Chr2,
    /// Chromosome 3
    Chr3,
    /// Chromosome 4
    Chr4,
    /// Chromosome 5
    Chr5,
    /// Chromosome 6
    Chr6,
    /// Chromosome 7
    Chr7,
    /// Chromosome 8
    Chr8,
    /// Chromosome 9
    Chr9,
    /// Chromosome 10
    Chr10,
    /// Chromosome 11
    Chr11,
    /// Chromosome 12
    Chr12,
    /// Chromosome 13
    Chr13,
    /// Chromosome 14
    Chr14,
    /// Chromosome 15
    Chr15,
    /// Chromosome 16
    Chr16,
    /// Chromosome 17
    Chr17,
    /// Chromosome 18
    Chr18,
    /// Chromosome 19
    Chr19,
    /// Chromosome 20
    Chr20,
    /// Chromosome 21
    Chr21,
    /// Chromosome 22
    Chr22,
    /// Mitochondrial DNA
    MT,
    /// Chromosome X
    X,
    /// Chromosome Y
    Y,
}
impl HumanContig {
    pub fn iter_all() -> [Self; 25] {
        [
            Self::Chr1,
            Self::Chr2,
            Self::Chr3,
            Self::Chr4,
            Self::Chr5,
            Self::Chr6,
            Self::Chr7,
            Self::Chr8,
            Self::Chr9,
            Self::Chr10,
            Self::Chr11,
            Self::Chr12,
            Self::Chr13,
            Self::Chr14,
            Self::Chr15,
            Self::Chr16,
            Self::Chr17,
            Self::Chr18,
            Self::Chr19,
            Self::Chr20,
            Self::Chr21,
            Self::Chr22,
            Self::MT,
            Self::X,
            Self::Y,
        ]
    }
    pub fn as_str(self) -> &'static str {
        match self {
            HumanContig::Chr1 => "1",
            HumanContig::Chr2 => "2",
            HumanContig::Chr3 => "3",
            HumanContig::Chr4 => "4",
            HumanContig::Chr5 => "5",
            HumanContig::Chr6 => "6",
            HumanContig::Chr7 => "7",
            HumanContig::Chr8 => "8",
            HumanContig::Chr9 => "9",
            HumanContig::Chr10 => "10",
            HumanContig::Chr11 => "11",
            HumanContig::Chr12 => "12",
            HumanContig::Chr13 => "13",
            HumanContig::Chr14 => "14",
            HumanContig::Chr15 => "15",
            HumanContig::Chr16 => "16",
            HumanContig::Chr17 => "17",
            HumanContig::Chr18 => "18",
            HumanContig::Chr19 => "19",
            HumanContig::Chr20 => "20",
            HumanContig::Chr21 => "21",
            HumanContig::Chr22 => "22",
            HumanContig::MT => "MT",
            HumanContig::X => "X",
            HumanContig::Y => "Y",
        }
    }
}

fn parse_human_contig(v: &str) -> Result<HumanContig, std::io::Error> {
    match v.trim() {
        "1" => Ok(HumanContig::Chr1),
        "2" => Ok(HumanContig::Chr2),
        "3" => Ok(HumanContig::Chr3),
        "4" => Ok(HumanContig::Chr4),
        "5" => Ok(HumanContig::Chr5),
        "6" => Ok(HumanContig::Chr6),
        "7" => Ok(HumanContig::Chr7),
        "8" => Ok(HumanContig::Chr8),
        "9" => Ok(HumanContig::Chr9),
        "10" => Ok(HumanContig::Chr10),
        "11" => Ok(HumanContig::Chr11),
        "12" => Ok(HumanContig::Chr12),
        "13" => Ok(HumanContig::Chr13),
        "14" => Ok(HumanContig::Chr14),
        "15" => Ok(HumanContig::Chr15),
        "16" => Ok(HumanContig::Chr16),
        "17" => Ok(HumanContig::Chr17),
        "18" => Ok(HumanContig::Chr18),
        "19" => Ok(HumanContig::Chr19),
        "20" => Ok(HumanContig::Chr20),
        "21" => Ok(HumanContig::Chr21),
        "22" => Ok(HumanContig::Chr22),
        "X" => Ok(HumanContig::X),
        "Y" => Ok(HumanContig::Y),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, v)),
    }
}

async fn get_latest_key(url: &'static str) -> std::io::Result<GwasCatalogResource> {
    let head = reqwest::Client::new()
        .head(url)
        .send()
        .await
        .map_err(reqwest_error)?;

    let file_name = utile::io::get_filename_from_headers(head.headers()).unwrap();
    let file_size = utile::io::get_filesize_from_headers(head.headers()).unwrap();

    Ok(GwasCatalogResource {
        url,
        key: file_name,
        size: file_size,
    })
}
