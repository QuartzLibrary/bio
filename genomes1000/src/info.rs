#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordInfo {
    /// Confidence interval around END for imprecise variants
    // CIEND,Number=2,Type=Integer,
    #[serde(rename = "CIEND")]
    ciend: [u64; 2],
    /// Confidence interval around POS for imprecise variants
    // CIPOS,Number=2,Type=Integer,
    #[serde(rename = "CIPOS")]
    cipos: [u64; 2],
    /// Source call set.
    // CS,Number=1,Type=String,
    #[serde(rename = "CS")]
    cs: String,
    /// End coordinate of this variant
    // END,Number=1,Type=Integer,
    #[serde(rename = "END")]
    end: u64,
    /// Imprecise pub structural variation
    // IMPRECISE,Number=0,Type=Flag,
    #[serde(rename = "IMPRECISE")]
    imprecise: bool,
    /// Merged calls.
    // MC,Number=.,Type=String,
    #[serde(rename = "MC")]
    mc: Vec<String>,
    /// Mobile element info of the form NAME,START,END<POLARITY; If there is only 5' OR 3' support for this call, will be NULL NULL for START and END
    // MEINFO,Number=4,Type=String,
    #[serde(rename = "MEINFO")]
    meinfo: [String; 4],
    /// Mitochondrial end coordinate of inserted sequence
    // MEND,Number=1,Type=Integer,
    #[serde(rename = "MEND")]
    mend: u64,
    /// Estimated length of mitochondrial insert
    // MLEN,Number=1,Type=Integer,
    #[serde(rename = "MLEN")]
    mlen: u64,
    /// Mitochondrial start coordinate of inserted sequence
    // MSTART,Number=1,Type=Integer,
    #[serde(rename = "MSTART")]
    mstart: u64,
    /// SV length. It is only calculated for structural variation MEIs. For other types of SVs; one may calculate the SV length by INFO:END-START+1, or by finding the difference between lengthes of REF and ALT alleles
    // SVLEN,Number=.,Type=Integer,
    #[serde(rename = "SVLEN")]
    svlen: Vec<u64>,
    /// Type of structural variant
    // SVTYPE,Number=1,Type=String,
    #[serde(rename = "SVTYPE")]
    svtype: String,
    /// Precise Target Site Duplication for bases, if unknown, value will be NULL
    // TSD,Number=1,Type=String,
    #[serde(rename = "TSD")]
    tsd: String,
    /// Total number of alternate alleles in called genotypes
    // AC,Number=A,Type=Integer,
    #[serde(rename = "AC")]
    ac: Vec<u64>,
    /// Estimated allele frequency in the range (0,1)
    // AF,Number=A,Type=Float,
    #[serde(rename = "AF")]
    af: Vec<f64>,
    /// Number of samples with data
    // NS,Number=1,Type=Integer,
    #[serde(rename = "NS")]
    ns: u64,
    /// Total number of alleles in called genotypes
    // AN,Number=1,Type=Integer,
    #[serde(rename = "AN")]
    an: u64,
    /// Allele frequency in the EAS populations calculated from AC and AN, in the range (0,1)
    // EAS_AF,Number=A,Type=Float,
    #[serde(rename = "EAS_AF")]
    eas_af: Vec<f64>,
    /// Allele frequency in the EUR populations calculated from AC and AN, in the range (0,1)
    // EUR_AF,Number=A,Type=Float,
    #[serde(rename = "EUR_AF")]
    eur_af: Vec<f64>,
    /// Allele frequency in the AFR populations calculated from AC and AN, in the range (0,1)
    // AFR_AF,Number=A,Type=Float,
    #[serde(rename = "AFR_AF")]
    afr_af: Vec<f64>,
    /// Allele frequency in the AMR populations calculated from AC and AN, in the range (0,1)
    // AMR_AF,Number=A,Type=Float,
    #[serde(rename = "AMR_AF")]
    amr_af: Vec<f64>,
    /// Allele frequency in the SAS populations calculated from AC and AN, in the range (0,1)
    // SAS_AF,Number=A,Type=Float,
    #[serde(rename = "SAS_AF")]
    sas_af: Vec<f64>,
    /// Total read depth; only low coverage data were counted towards the DP, exome data were not used
    // DP,Number=1,Type=Integer,
    #[serde(rename = "DP")]
    dp: u64,
    /// Ancestral Allele. Format: AA|REF|ALT|IndelType. AA: Ancestral allele, REF:Reference Allele, ALT:Alternate Allele, IndelType:Type of Indel (REF, ALT and IndelType are only defined for indels)
    // AA,Number=1,Type=String,
    #[serde(rename = "AA")]
    aa: String,
    /// indicates what type of variant the line represents
    // VT,Number=.,Type=String,
    #[serde(rename = "VT")]
    vt: Vec<String>,
    /// indicates whether a variant is within the exon pull down target boundaries
    // EX_TARGET,Number=0,Type=Flag,
    #[serde(rename = "EX_TARGET")]
    ex_target: bool,
    /// indicates whether a site is multi-allelic
    // MULTI_ALLELIC,Number=0,Type=Flag,
    #[serde(rename = "MULTI_ALLELIC")]
    multi_allelic: bool,
}
