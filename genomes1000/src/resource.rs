use url::Url;
use utile::resource::{RawResource, UrlResource};

use crate::contig::GRCh38Contig;

const BASE_URL: &str = "https://ftp.1000genomes.ebi.ac.uk/";
const REFERENCE_GENOME: &str = "vol1/ftp/technical/reference/GRCh38_reference_genome/GRCh38_full_analysis_set_plus_decoy_hla.fa";

// Picked the same files as:
// https://www.cog-genomics.org/plink/2.0/resources#phase3_1kg

// https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20220422_3202_phased_SNV_INDEL_SV/
// https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20220422_3202_phased_SNV_INDEL_SV/1kGP_high_coverage_Illumina.chr1.filtered.SNV_INDEL_SV_phased_panel.vcf.gz
const CHR_BASE: &str =
    "vol1/ftp/data_collections/1000G_2504_high_coverage/working/20220422_3202_phased_SNV_INDEL_SV";
const CHR_X_NAME: &str =
    "1kGP_high_coverage_Illumina.chrX.filtered.SNV_INDEL_SV_phased_panel.v2.vcf.gz";

// https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20201028_3202_raw_GT_with_annot/
const CHR_ALT_BASE: &str =
    "vol1/ftp/data_collections/1000G_2504_high_coverage/working/20201028_3202_raw_GT_with_annot";
const CHR_Y_NAME: &str =
    "20201028_CCDG_14151_B01_GRM_WGS_2020-08-05_chrY.recalibrated_variants.vcf.gz";
const OTHER_NAME: &str =
    "20201028_CCDG_14151_B01_GRM_WGS_2020-08-05_others.recalibrated_variants.vcf.gz";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Genomes1000Resource {
    key: String,
}
impl Genomes1000Resource {
    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn grch38_reference_genome() -> Self {
        Self::new(REFERENCE_GENOME.to_owned())
    }
    pub fn grch38_reference_genome_index() -> Self {
        Self::new(format!("{REFERENCE_GENOME}.fai"))
    }

    pub fn into_bcf(self) -> Self {
        if let Some(name) = self.key.strip_suffix(".vcf.gz") {
            Self::new(format!("{name}.bcf"))
        } else if let Some(name) = self.key.strip_suffix(".vcf.gz.tbi") {
            Self::new(format!("{name}.bcf.csi"))
        } else {
            unreachable!()
        }
    }

    pub fn high_coverage_pedigree() -> Self {
        Self::new("vol1/ftp/data_collections/1000G_2504_high_coverage/20130606_g1k_3202_samples_ped_population.txt".to_owned())
    }

    pub fn high_coverage_genotypes_contig_vcf(contig: GRCh38Contig) -> Self {
        match contig {
            GRCh38Contig::CHR1 => Self::high_coverage_genotypes_chr_vcf(1),
            GRCh38Contig::CHR2 => Self::high_coverage_genotypes_chr_vcf(2),
            GRCh38Contig::CHR3 => Self::high_coverage_genotypes_chr_vcf(3),
            GRCh38Contig::CHR4 => Self::high_coverage_genotypes_chr_vcf(4),
            GRCh38Contig::CHR5 => Self::high_coverage_genotypes_chr_vcf(5),
            GRCh38Contig::CHR6 => Self::high_coverage_genotypes_chr_vcf(6),
            GRCh38Contig::CHR7 => Self::high_coverage_genotypes_chr_vcf(7),
            GRCh38Contig::CHR8 => Self::high_coverage_genotypes_chr_vcf(8),
            GRCh38Contig::CHR9 => Self::high_coverage_genotypes_chr_vcf(9),
            GRCh38Contig::CHR10 => Self::high_coverage_genotypes_chr_vcf(10),
            GRCh38Contig::CHR11 => Self::high_coverage_genotypes_chr_vcf(11),
            GRCh38Contig::CHR12 => Self::high_coverage_genotypes_chr_vcf(12),
            GRCh38Contig::CHR13 => Self::high_coverage_genotypes_chr_vcf(13),
            GRCh38Contig::CHR14 => Self::high_coverage_genotypes_chr_vcf(14),
            GRCh38Contig::CHR15 => Self::high_coverage_genotypes_chr_vcf(15),
            GRCh38Contig::CHR16 => Self::high_coverage_genotypes_chr_vcf(16),
            GRCh38Contig::CHR17 => Self::high_coverage_genotypes_chr_vcf(17),
            GRCh38Contig::CHR18 => Self::high_coverage_genotypes_chr_vcf(18),
            GRCh38Contig::CHR19 => Self::high_coverage_genotypes_chr_vcf(19),
            GRCh38Contig::CHR20 => Self::high_coverage_genotypes_chr_vcf(20),
            GRCh38Contig::CHR21 => Self::high_coverage_genotypes_chr_vcf(21),
            GRCh38Contig::CHR22 => Self::high_coverage_genotypes_chr_vcf(22),

            GRCh38Contig::X => Self::high_coverage_genotypes_x_chr_vcf(),
            GRCh38Contig::Y => Self::high_coverage_genotypes_y_chr_vcf(),
            _ => Self::high_coverage_genotypes_other_chr_vcf(),
        }
    }
    pub fn high_coverage_genotypes_contig_vcf_index(contig: GRCh38Contig) -> Self {
        match contig {
            GRCh38Contig::CHR1 => Self::high_coverage_genotypes_chr_vcf_index(1),
            GRCh38Contig::CHR2 => Self::high_coverage_genotypes_chr_vcf_index(2),
            GRCh38Contig::CHR3 => Self::high_coverage_genotypes_chr_vcf_index(3),
            GRCh38Contig::CHR4 => Self::high_coverage_genotypes_chr_vcf_index(4),
            GRCh38Contig::CHR5 => Self::high_coverage_genotypes_chr_vcf_index(5),
            GRCh38Contig::CHR6 => Self::high_coverage_genotypes_chr_vcf_index(6),
            GRCh38Contig::CHR7 => Self::high_coverage_genotypes_chr_vcf_index(7),
            GRCh38Contig::CHR8 => Self::high_coverage_genotypes_chr_vcf_index(8),
            GRCh38Contig::CHR9 => Self::high_coverage_genotypes_chr_vcf_index(9),
            GRCh38Contig::CHR10 => Self::high_coverage_genotypes_chr_vcf_index(10),
            GRCh38Contig::CHR11 => Self::high_coverage_genotypes_chr_vcf_index(11),
            GRCh38Contig::CHR12 => Self::high_coverage_genotypes_chr_vcf_index(12),
            GRCh38Contig::CHR13 => Self::high_coverage_genotypes_chr_vcf_index(13),
            GRCh38Contig::CHR14 => Self::high_coverage_genotypes_chr_vcf_index(14),
            GRCh38Contig::CHR15 => Self::high_coverage_genotypes_chr_vcf_index(15),
            GRCh38Contig::CHR16 => Self::high_coverage_genotypes_chr_vcf_index(16),
            GRCh38Contig::CHR17 => Self::high_coverage_genotypes_chr_vcf_index(17),
            GRCh38Contig::CHR18 => Self::high_coverage_genotypes_chr_vcf_index(18),
            GRCh38Contig::CHR19 => Self::high_coverage_genotypes_chr_vcf_index(19),
            GRCh38Contig::CHR20 => Self::high_coverage_genotypes_chr_vcf_index(20),
            GRCh38Contig::CHR21 => Self::high_coverage_genotypes_chr_vcf_index(21),
            GRCh38Contig::CHR22 => Self::high_coverage_genotypes_chr_vcf_index(22),

            GRCh38Contig::X => Self::high_coverage_genotypes_x_chr_vcf_index(),
            GRCh38Contig::Y => Self::high_coverage_genotypes_y_chr_vcf_index(),
            _ => Self::high_coverage_genotypes_other_chr_vcf_index(),
        }
    }

    fn high_coverage_genotypes_x_chr_vcf() -> Self {
        Self::new(format!("{CHR_BASE}/{CHR_X_NAME}"))
    }
    fn high_coverage_genotypes_x_chr_vcf_index() -> Self {
        Self::new(format!("{CHR_BASE}/{CHR_X_NAME}.tbi"))
    }

    fn high_coverage_genotypes_y_chr_vcf() -> Self {
        Self::new(format!("{CHR_ALT_BASE}/{CHR_Y_NAME}"))
    }
    fn high_coverage_genotypes_y_chr_vcf_index() -> Self {
        Self::new(format!("{CHR_ALT_BASE}/{CHR_Y_NAME}.tbi"))
    }

    fn high_coverage_genotypes_other_chr_vcf() -> Self {
        Self::new(format!("{CHR_ALT_BASE}/{OTHER_NAME}"))
    }
    fn high_coverage_genotypes_other_chr_vcf_index() -> Self {
        Self::new(format!("{CHR_ALT_BASE}/{OTHER_NAME}.tbi"))
    }

    fn high_coverage_genotypes_chr_vcf(chr: usize) -> Self {
        assert!((1..=22).contains(&chr));
        Self::new(format!(
            "{CHR_BASE}/1kGP_high_coverage_Illumina.chr{chr}.filtered.SNV_INDEL_SV_phased_panel.vcf.gz"
        ))
    }
    fn high_coverage_genotypes_chr_vcf_index(chr: usize) -> Self {
        assert!((1..=22).contains(&chr));
        Self::new(format!(
            "{CHR_BASE}/1kGP_high_coverage_Illumina.chr{chr}.filtered.SNV_INDEL_SV_phased_panel.vcf.gz.tbi"
        ))
    }

    pub fn url(&self) -> Url {
        let key = &self.key;
        Url::parse(&format!("{BASE_URL}{key}")).unwrap()
    }

    fn url_resource(&self) -> UrlResource {
        UrlResource::new(self.url()).unwrap()
    }
}
impl RawResource for Genomes1000Resource {
    const NAMESPACE: &'static str = "1000genomes";

    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<utile::resource::Compression> {
        if self.key == old::REFERENCE_GENOME {
            Some(utile::resource::Compression::Gzip)
        } else {
            utile::resource::Compression::infer(&self.key)
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
    use crate::contig::GRCh37Contig;

    use super::*;

    pub(super) const REFERENCE_GENOME: &str = "vol1/ftp/technical/reference/human_g1k_v37.fasta.gz";
    const REFERENCE_GENOME_INDEX: &str = "vol1/ftp/technical/reference/human_g1k_v37.fasta.fai";

    // const TODO: &str = "vol1/ftp/release/20130502/ALL.chr{number}.phase3_shapeit2_mvncall_integrated_v5b.20130502.genotypes.vcf.gz";
    const CHR_BASE: &str = "vol1/ftp/release/20130502";
    const CHR_X_NAME: &str =
        "ALL.chrX.phase3_shapeit2_mvncall_integrated_v1c.20130502.genotypes.vcf.gz";
    const CHR_Y_NAME: &str = "ALL.chrY.phase3_integrated_v2b.20130502.genotypes.vcf.gz";
    const MT_NAME: &str = "ALL.chrMT.phase3_callmom-v0_4.20130502.genotypes.vcf.gz";
    const WGS_NAME: &str = "ALL.wgs.phase3_shapeit2_mvncall_integrated_v5c.20130502.sites.vcf.gz";

    impl Genomes1000Resource {
        pub fn old_grch37_reference_genome() -> Self {
            Self::new(REFERENCE_GENOME.to_owned())
        }
        pub fn old_grch37_reference_genome_index() -> Self {
            Self::new(REFERENCE_GENOME_INDEX.to_owned())
        }

        pub fn old_phase_3_contig_vcf(contig: GRCh37Contig) -> Option<Self> {
            Some(match contig {
                GRCh37Contig::CHR1 => Self::old_phase_3_chr_vcf(1),
                GRCh37Contig::CHR2 => Self::old_phase_3_chr_vcf(2),
                GRCh37Contig::CHR3 => Self::old_phase_3_chr_vcf(3),
                GRCh37Contig::CHR4 => Self::old_phase_3_chr_vcf(4),
                GRCh37Contig::CHR5 => Self::old_phase_3_chr_vcf(5),
                GRCh37Contig::CHR6 => Self::old_phase_3_chr_vcf(6),
                GRCh37Contig::CHR7 => Self::old_phase_3_chr_vcf(7),
                GRCh37Contig::CHR8 => Self::old_phase_3_chr_vcf(8),
                GRCh37Contig::CHR9 => Self::old_phase_3_chr_vcf(9),
                GRCh37Contig::CHR10 => Self::old_phase_3_chr_vcf(10),
                GRCh37Contig::CHR11 => Self::old_phase_3_chr_vcf(11),
                GRCh37Contig::CHR12 => Self::old_phase_3_chr_vcf(12),
                GRCh37Contig::CHR13 => Self::old_phase_3_chr_vcf(13),
                GRCh37Contig::CHR14 => Self::old_phase_3_chr_vcf(14),
                GRCh37Contig::CHR15 => Self::old_phase_3_chr_vcf(15),
                GRCh37Contig::CHR16 => Self::old_phase_3_chr_vcf(16),
                GRCh37Contig::CHR17 => Self::old_phase_3_chr_vcf(17),
                GRCh37Contig::CHR18 => Self::old_phase_3_chr_vcf(18),
                GRCh37Contig::CHR19 => Self::old_phase_3_chr_vcf(19),
                GRCh37Contig::CHR20 => Self::old_phase_3_chr_vcf(20),
                GRCh37Contig::CHR21 => Self::old_phase_3_chr_vcf(21),
                GRCh37Contig::CHR22 => Self::old_phase_3_chr_vcf(22),

                GRCh37Contig::X => Self::old_phase_3_x_chr_vcf(),
                GRCh37Contig::Y => Self::old_phase_3_y_chr_vcf(),
                GRCh37Contig::MT => Self::old_phase_3_mt_chr_vcf(),
                _ => return None,
            })
        }
        pub fn old_phase_3_contig_vcf_index(contig: GRCh37Contig) -> Option<Self> {
            Some(match contig {
                GRCh37Contig::CHR1 => Self::old_phase_3_chr_vcf_index(1),
                GRCh37Contig::CHR2 => Self::old_phase_3_chr_vcf_index(2),
                GRCh37Contig::CHR3 => Self::old_phase_3_chr_vcf_index(3),
                GRCh37Contig::CHR4 => Self::old_phase_3_chr_vcf_index(4),
                GRCh37Contig::CHR5 => Self::old_phase_3_chr_vcf_index(5),
                GRCh37Contig::CHR6 => Self::old_phase_3_chr_vcf_index(6),
                GRCh37Contig::CHR7 => Self::old_phase_3_chr_vcf_index(7),
                GRCh37Contig::CHR8 => Self::old_phase_3_chr_vcf_index(8),
                GRCh37Contig::CHR9 => Self::old_phase_3_chr_vcf_index(9),
                GRCh37Contig::CHR10 => Self::old_phase_3_chr_vcf_index(10),
                GRCh37Contig::CHR11 => Self::old_phase_3_chr_vcf_index(11),
                GRCh37Contig::CHR12 => Self::old_phase_3_chr_vcf_index(12),
                GRCh37Contig::CHR13 => Self::old_phase_3_chr_vcf_index(13),
                GRCh37Contig::CHR14 => Self::old_phase_3_chr_vcf_index(14),
                GRCh37Contig::CHR15 => Self::old_phase_3_chr_vcf_index(15),
                GRCh37Contig::CHR16 => Self::old_phase_3_chr_vcf_index(16),
                GRCh37Contig::CHR17 => Self::old_phase_3_chr_vcf_index(17),
                GRCh37Contig::CHR18 => Self::old_phase_3_chr_vcf_index(18),
                GRCh37Contig::CHR19 => Self::old_phase_3_chr_vcf_index(19),
                GRCh37Contig::CHR20 => Self::old_phase_3_chr_vcf_index(20),
                GRCh37Contig::CHR21 => Self::old_phase_3_chr_vcf_index(21),
                GRCh37Contig::CHR22 => Self::old_phase_3_chr_vcf_index(22),

                GRCh37Contig::X => Self::old_phase_3_x_chr_vcf_index(),
                GRCh37Contig::Y => Self::old_phase_3_y_chr_vcf_index(),
                GRCh37Contig::MT => Self::old_phase_3_mt_chr_vcf_index(),
                _ => return None,
            })
        }

        pub fn old_phase_3_wgs_sites_vcf() -> Self {
            Self::new(format!("{CHR_BASE}/{WGS_NAME}"))
        }
        pub fn old_phase_3_wgs_sites_vcf_index() -> Self {
            Self::new(format!("{CHR_BASE}/{WGS_NAME}.tbi"))
        }

        fn old_phase_3_x_chr_vcf() -> Self {
            Self::new(format!("{CHR_BASE}/{CHR_X_NAME}"))
        }
        fn old_phase_3_x_chr_vcf_index() -> Self {
            Self::new(format!("{CHR_BASE}/{CHR_X_NAME}.tbi"))
        }

        fn old_phase_3_y_chr_vcf() -> Self {
            Self::new(format!("{CHR_BASE}/{CHR_Y_NAME}"))
        }
        fn old_phase_3_y_chr_vcf_index() -> Self {
            Self::new(format!("{CHR_BASE}/{CHR_Y_NAME}.tbi"))
        }

        fn old_phase_3_mt_chr_vcf() -> Self {
            Self::new(format!("{CHR_BASE}/{MT_NAME}"))
        }
        fn old_phase_3_mt_chr_vcf_index() -> Self {
            Self::new(format!("{CHR_BASE}/{MT_NAME}.tbi"))
        }

        fn old_phase_3_chr_vcf(chr: usize) -> Self {
            assert!((1..=22).contains(&chr));
            Self::new(format!(
                "{CHR_BASE}/ALL.chr{chr}.phase3_shapeit2_mvncall_integrated_v5b.20130502.genotypes.vcf.gz"
            ))
        }
        fn old_phase_3_chr_vcf_index(chr: usize) -> Self {
            assert!((1..=22).contains(&chr));
            Self::new(format!(
                "{CHR_BASE}/ALL.chr{chr}.phase3_shapeit2_mvncall_integrated_v5b.20130502.genotypes.vcf.gz.tbi"
            ))
        }
    }
}
