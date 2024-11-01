use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use url::Url;
use utile::io::FromUtf8Bytes;

// Picked the same files as:
// https://www.cog-genomics.org/plink/2.0/resources#phase3_1kg

// https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20220422_3202_phased_SNV_INDEL_SV/
// https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20220422_3202_phased_SNV_INDEL_SV/1kGP_high_coverage_Illumina.chr1.filtered.SNV_INDEL_SV_phased_panel.vcf.gz
const BASE_DOWNLOAD_URL: &str = "https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20220422_3202_phased_SNV_INDEL_SV";

// https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20201028_3202_raw_GT_with_annot/
const Y_DOWNLOAD_URL: &str = "https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20201028_3202_raw_GT_with_annot/20201028_CCDG_14151_B01_GRM_WGS_2020-08-05_chrY.recalibrated_variants.vcf.gz";
pub(super) const OTHER_DOWNLOAD_URL: &str = "https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/data_collections/1000G_2504_high_coverage/working/20201028_3202_raw_GT_with_annot/20201028_CCDG_14151_B01_GRM_WGS_2020-08-05_others.recalibrated_variants.vcf.gz";

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B38Contig {
    Chr1,
    Chr2,
    Chr3,
    Chr4,
    Chr5,
    Chr6,
    Chr7,
    Chr8,
    Chr9,
    Chr10,
    Chr11,
    Chr12,
    Chr13,
    Chr14,
    Chr15,
    Chr16,
    Chr17,
    Chr18,
    Chr19,
    Chr20,
    Chr21,
    Chr22,
    MT,
    X,
    Y,
    Other(String),
}
impl B38Contig {
    pub fn new(i: u8) -> Option<Self> {
        format!("chr{i}").parse().ok()
    }
    pub fn download_url(self) -> Url {
        match self {
            B38Contig::Chr1
            | B38Contig::Chr2
            | B38Contig::Chr3
            | B38Contig::Chr4
            | B38Contig::Chr5
            | B38Contig::Chr6
            | B38Contig::Chr7
            | B38Contig::Chr8
            | B38Contig::Chr9
            | B38Contig::Chr10
            | B38Contig::Chr11
            | B38Contig::Chr12
            | B38Contig::Chr13
            | B38Contig::Chr14
            | B38Contig::Chr15
            | B38Contig::Chr16
            | B38Contig::Chr17
            | B38Contig::Chr18
            | B38Contig::Chr19
            | B38Contig::Chr20
            | B38Contig::Chr21
            | B38Contig::Chr22  => {
                return format!("{BASE_DOWNLOAD_URL}/1kGP_high_coverage_Illumina.{self}.filtered.SNV_INDEL_SV_phased_panel.vcf.gz")
                    .parse()
                    .unwrap()
            }
            B38Contig::X => {
                return format!("{BASE_DOWNLOAD_URL}/1kGP_high_coverage_Illumina.{self}.filtered.SNV_INDEL_SV_phased_panel.v2.vcf.gz")
                    .parse()
                    .unwrap()
            }

            B38Contig::MT | B38Contig::Other(_)=> OTHER_DOWNLOAD_URL,
            B38Contig::Y => Y_DOWNLOAD_URL,
        }
        .parse()
        .unwrap()
    }
    pub fn iter() -> [Self; 25] {
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
}

impl fmt::Display for B38Contig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Chr1 => "chr1",
            Self::Chr2 => "chr2",
            Self::Chr3 => "chr3",
            Self::Chr4 => "chr4",
            Self::Chr5 => "chr5",
            Self::Chr6 => "chr6",
            Self::Chr7 => "chr7",
            Self::Chr8 => "chr8",
            Self::Chr9 => "chr9",
            Self::Chr10 => "chr10",
            Self::Chr11 => "chr11",
            Self::Chr12 => "chr12",
            Self::Chr13 => "chr13",
            Self::Chr14 => "chr14",
            Self::Chr15 => "chr15",
            Self::Chr16 => "chr16",
            Self::Chr17 => "chr17",
            Self::Chr18 => "chr18",
            Self::Chr19 => "chr19",
            Self::Chr20 => "chr20",
            Self::Chr21 => "chr21",
            Self::Chr22 => "chr22",

            Self::MT => "chrM",
            Self::X => "chrX",
            Self::Y => "chrY",

            Self::Other(v) => v,
        };
        f.write_str(v)
    }
}
impl FromStr for B38Contig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes()).map_err(|_| s.to_owned())
    }
}

impl FromUtf8Bytes for B38Contig {
    type Err = std::string::FromUtf8Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        Ok(match bytes {
            b"chr1" => Self::Chr1,
            b"chr2" => Self::Chr2,
            b"chr3" => Self::Chr3,
            b"chr4" => Self::Chr4,
            b"chr5" => Self::Chr5,
            b"chr6" => Self::Chr6,
            b"chr7" => Self::Chr7,
            b"chr8" => Self::Chr8,
            b"chr9" => Self::Chr9,
            b"chr10" => Self::Chr10,
            b"chr11" => Self::Chr11,
            b"chr12" => Self::Chr12,
            b"chr13" => Self::Chr13,
            b"chr14" => Self::Chr14,
            b"chr15" => Self::Chr15,
            b"chr16" => Self::Chr16,
            b"chr17" => Self::Chr17,
            b"chr18" => Self::Chr18,
            b"chr19" => Self::Chr19,
            b"chr20" => Self::Chr20,
            b"chr21" => Self::Chr21,
            b"chr22" => Self::Chr22,
            b"chrM" => Self::MT,
            b"chrX" => Self::X,
            b"chrY" => Self::Y,
            _ => return String::from_utf8(bytes.to_vec()).map(Self::Other),
        })
    }
}
