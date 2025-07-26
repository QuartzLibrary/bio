use ids::rs::RsId;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use biocore::{
    dna::DnaSequence,
    location::{GenomePosition, GenomeRange},
};

use crate::{Allele, HarmonizedSource, HarmonizedStudyAssociation, ImputationMethod};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimplifiedHarmonizedStudyAssociation<Contig> {
    /// The SNP’s rs ID.
    pub rs_id: Option<RsId>,
    /// Chromosome name
    pub chr: Contig,
    /// Chromosome position
    pub pos: u64,

    /// The allele that's dosage is counted (e.g. {0, 1, 2}) and multiplied by
    /// the variant's weight (effect_weight) when calculating score. The effect
    /// allele is also known as the 'risk allele'. Note: this does not
    /// necessarily need to correspond to the minor allele/alternative allele.
    pub effect_allele: DnaSequence,
    /// The other allele(s) at the loci. Note: this does not necessarily need to
    /// correspond to the reference allele.
    ///
    /// This field might have been harmonized.
    ///
    /// If only the effect_allele is given we attempt to infer the
    /// non-effect/other allele(s) using Ensembl/dbSNP alleles.
    /// See [docs::OTHER_ALLELE_INFERRED] for what kind of values are present in [Allele::Other].
    pub other_allele: Option<Allele>,

    /// This is kept in for loci where the variant may be referenced by the gene
    /// (APOE e4). It is also common (usually in smaller PGS) to see the
    /// variants named according to the genes they impact.
    pub locus_name: Option<String>,

    pub kind: AssociationKind,

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

    pub effect: Effect,

    /// Reported effect allele frequency, if the associated locus is a haplotype
    /// then haplotype frequency will be extracted.
    pub allelefrequency_effect: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    pub allelefrequency_effect_european: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    pub allelefrequency_effect_asian: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    pub allelefrequency_effect_african: Option<NotNan<f64>>,
    /// Reported effect allele frequency in a specific population (described by the authors).
    pub allelefrequency_effect_hispanic: Option<NotNan<f64>>,

    pub variant_type: Option<String>,

    // -------------------------------------------
    // Harmonized-only fields start here
    // -------------------------------------------
    /// Provider of the harmonized variant information
    ///
    /// Data source of the variant position. Options include: ENSEMBL, liftover,
    /// author-reported (if being harmonized to the same build).
    pub source: HarmonizedSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum AssociationKind {
    Normal,
    /// This is a TRUE/FALSE variable that flags whether the effect allele is a
    /// haplotype/diplotype rather than a single SNP. Constituent SNPs in the
    /// haplotype are semi-colon separated.
    Haplotype,
    /// This is a TRUE/FALSE variable that flags whether the effect allele is a
    /// haplotype/diplotype rather than a single SNP. Constituent SNPs in the
    /// haplotype are semi-colon separated.
    Diplotype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Effect {
    Additive {
        effect_weight: NotNan<f64>,
        // Either OR, HR, or neither. Never both.
        or: Option<NotNan<f64>>,
        hr: Option<NotNan<f64>>,
    },

    /// Weights that are specific to different dosages of the effect_allele
    /// (e.g. {0, 1, 2} copies) can also be reported when the the contribution
    /// of the variants to the score is not encoded as additive, dominant, or
    /// recessive. In this case three columns are added corresponding to which
    /// variant weight should be applied for each dosage, where the column name
    /// is formated as dosage_#_weight where the # sign indicates the number of
    /// effect_allele copies.
    DosageSpecific {
        dosage_0_weight: NotNan<f64>,
        dosage_1_weight: NotNan<f64>,
        dosage_2_weight: NotNan<f64>,
        effect_weight: Option<NotNan<f64>>,
        or: Option<NotNan<f64>>,
    },

    /// The weight should be added to the PGS sum if there is at least 1 copy of the
    /// effect allele (e.g. it is a dominant allele).
    Dominant { effect_weight: NotNan<f64> },

    /// The weight should be added to the PGS sum only if there are 2 copies of the
    /// effect allele (e.g. it is a recessive allele).
    Recessive { effect_weight: NotNan<f64> },
    // /// The weight should be multiplied with the dosage of more than one variant.
    // /// Interactions are demarcated with a _x_ between entries for each of the
    // /// variants present in the interaction.
    // Interaction { effect_weight: NotNan<f64> },
}

pub enum Ploidy {
    Diploid,
    Haploid,
}

impl<Contig> SimplifiedHarmonizedStudyAssociation<Contig> {
    pub fn new(
        association: HarmonizedStudyAssociation,
        contig: impl FnOnce(String) -> Contig,
    ) -> Result<Self, SimplificationError> {
        let HarmonizedStudyAssociation {
            rs_id,
            chr_name: _,
            chr_position: _,
            effect_allele,
            other_allele,
            locus_name,
            is_haplotype,
            is_diplotype,
            imputation_method,
            variant_description,
            inclusion_criteria,
            effect_weight: _,
            is_interaction: _,
            is_dominant: _,
            is_recessive: _,
            dosage_0_weight: _,
            dosage_1_weight: _,
            dosage_2_weight: _,
            or: _,
            hr: _,
            allelefrequency_effect,
            allelefrequency_effect_european,
            allelefrequency_effect_asian,
            allelefrequency_effect_african,
            allelefrequency_effect_hispanic,
            variant_type,
            source,
            hm_rs_id,
            chr,
            pos,
            infer_other_allele,
            match_chr: _,
            match_pos: _,
        } = association.clone();

        if chr.is_empty() {
            return Err(SimplificationError::NoChromosome);
        }

        Ok(Self {
            rs_id: hm_rs_id.or_else(|| rs_id.as_ref()?.parse().ok()),
            chr: contig(chr),
            pos: pos.ok_or(SimplificationError::NoPosition)?,
            effect_allele: match effect_allele {
                Allele::Sequence(sequence) if sequence.is_empty() => {
                    Err(SimplificationError::EmptyEffectAllele)?
                }
                Allele::Insertion | Allele::Other(_) => {
                    Err(SimplificationError::InvalidEffectAllele(effect_allele))?
                }

                Allele::Sequence(sequence) => sequence,
            },
            other_allele: infer_other_allele.or(other_allele),
            locus_name,
            kind: match (is_haplotype, is_diplotype) {
                (None | Some(false), None | Some(false)) => AssociationKind::Normal,
                (None | Some(false), Some(true)) => AssociationKind::Diplotype,
                (Some(true), None | Some(false)) => AssociationKind::Haplotype,
                (Some(true), Some(true)) => unreachable!(),
            },
            imputation_method,
            variant_description,
            inclusion_criteria,
            effect: Effect::from_association(association)?,
            allelefrequency_effect,
            allelefrequency_effect_european,
            allelefrequency_effect_asian,
            allelefrequency_effect_african,
            allelefrequency_effect_hispanic,
            variant_type,
            source,
        })
    }

    pub fn at(&self) -> GenomePosition<Contig>
    where
        Contig: Clone,
    {
        GenomePosition {
            name: self.chr.clone(),
            at: self.pos - 1,
        }
    }
    pub fn at_range(&self) -> GenomeRange<Contig>
    where
        Contig: Clone,
    {
        GenomeRange {
            name: self.chr.clone(),
            at: self.pos - 1..(self.pos - 1 + u64::try_from(self.effect_allele.len()).unwrap()),
        }
    }

    pub fn max_edit(&self, dosage: u8, ploidy: u8) -> NotNan<f64> {
        Ord::max(
            self.edit_force_alt(dosage, ploidy),
            self.edit_force_ref(dosage, ploidy),
        )
    }
    pub fn min_edit(&self, dosage: u8, ploidy: u8) -> NotNan<f64> {
        Ord::min(
            self.edit_force_alt(dosage, ploidy),
            self.edit_force_ref(dosage, ploidy),
        )
    }

    pub fn edit_force_alt(&self, dosage: u8, ploidy: u8) -> NotNan<f64> {
        self.effect.score(ploidy, ploidy) - self.effect.score(dosage, ploidy)
    }
    pub fn edit_force_ref(&self, dosage: u8, ploidy: u8) -> NotNan<f64> {
        self.effect.score(0, ploidy) - self.effect.score(dosage, ploidy)
    }
}

impl Effect {
    pub fn from_association(
        association: crate::HarmonizedStudyAssociation,
    ) -> Result<Self, SimplificationError> {
        if let Some(effect) = try_dosage_specific(association.clone()) {
            return Ok(effect);
        }

        let crate::HarmonizedStudyAssociation {
            rs_id: _,
            chr_name: _,
            chr_position: _,
            effect_allele: _,
            other_allele: _,
            locus_name: _,
            is_haplotype: _,
            is_diplotype: _,
            imputation_method: _,
            variant_description: _,
            inclusion_criteria: _,
            effect_weight,
            is_interaction,
            is_dominant,
            is_recessive,
            dosage_0_weight,
            dosage_1_weight,
            dosage_2_weight,
            or,
            hr,
            allelefrequency_effect: _,
            allelefrequency_effect_european: _,
            allelefrequency_effect_asian: _,
            allelefrequency_effect_african: _,
            allelefrequency_effect_hispanic: _,
            variant_type: _,
            source: _,
            hm_rs_id: _,
            chr: _,
            pos: _,
            infer_other_allele: _,
            match_chr: _,
            match_pos: _,
        } = association; // Exhaustiveness check of all fields

        let effect_weight = effect_weight.unwrap();

        assert_eq!(None, dosage_0_weight);
        assert_eq!(None, dosage_1_weight);
        assert_eq!(None, dosage_2_weight);

        if let Some(true) = is_interaction {
            assert_eq!(None, is_dominant);
            assert_eq!(None, is_recessive);
            assert_eq!(None, hr);
            assert_eq!(None, or);
            // return Self::Interaction { effect_weight };
            return Err(SimplificationError::Interaction);
        }

        Ok(match (is_dominant, is_recessive) {
            (None, None) => Self::Additive {
                effect_weight,
                or,
                hr,
            },
            (None, Some(true)) => {
                assert_eq!(None, is_interaction);
                assert_eq!(None, hr);
                assert_eq!(None, or);
                Self::Recessive { effect_weight }
            }
            (None, Some(false)) => {
                assert!(matches!(is_interaction, None | Some(false)));
                assert_eq!(None, hr);
                assert_eq!(None, or);
                Self::Additive {
                    effect_weight,
                    or: None,
                    hr: None,
                }
            }
            (Some(_), None) => unreachable!(),
            (Some(false), Some(false)) => {
                assert_eq!(None, is_interaction);
                assert_eq!(None, hr);
                assert_eq!(None, or);
                Self::Additive {
                    effect_weight,
                    or: None,
                    hr: None,
                }
            }
            (Some(false), Some(true)) => {
                assert_eq!(None, is_interaction);
                assert_eq!(None, hr);
                assert_eq!(None, or);
                Self::Recessive { effect_weight }
            }
            (Some(true), Some(false)) => {
                assert_eq!(None, is_interaction);
                assert_eq!(None, hr);
                assert_eq!(None, or);
                Self::Dominant { effect_weight }
            }
            (Some(true), Some(true)) => unreachable!(),
        })
    }

    pub fn score(self, dosage: u8, ploidy: u8) -> NotNan<f64> {
        match self {
            Self::Additive {
                effect_weight,
                or: _,
                hr: _,
            } => NotNan::new(effect_weight * dosage as f64).unwrap(),
            Self::DosageSpecific {
                dosage_0_weight,
                dosage_1_weight,
                dosage_2_weight,
                effect_weight: _,
                or: _,
            } => match dosage {
                0 => dosage_0_weight,
                1 => dosage_1_weight,
                2 => dosage_2_weight,
                _ => unreachable!(),
            },
            Self::Dominant { effect_weight } => {
                assert_eq!(ploidy, 2);
                if dosage > 0 {
                    effect_weight
                } else {
                    NotNan::new(0.0).unwrap()
                }
            }
            Self::Recessive { effect_weight } => {
                assert_eq!(ploidy, 2);
                if dosage == 2 {
                    effect_weight
                } else {
                    NotNan::new(0.0).unwrap()
                }
            } // Self::Interaction { effect_weight: _ } => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SimplificationError {
    NoChromosome,
    NoPosition,
    EmptyEffectAllele,
    InvalidEffectAllele(Allele),
    Interaction,
}

fn try_dosage_specific(association: crate::HarmonizedStudyAssociation) -> Option<Effect> {
    let crate::HarmonizedStudyAssociation {
        rs_id: _,
        chr_name: _,
        chr_position: _,
        effect_allele: _,
        other_allele: _,
        locus_name: _,
        is_haplotype: _,
        is_diplotype: _,
        imputation_method: _,
        variant_description: _,
        inclusion_criteria: _,
        effect_weight,
        is_interaction,
        is_dominant,
        is_recessive,
        dosage_0_weight,
        dosage_1_weight,
        dosage_2_weight,
        or,
        hr,
        allelefrequency_effect: _,
        allelefrequency_effect_european: _,
        allelefrequency_effect_asian: _,
        allelefrequency_effect_african: _,
        allelefrequency_effect_hispanic: _,
        variant_type: _,
        source: _,
        hm_rs_id: _,
        chr: _,
        pos: _,
        infer_other_allele: _,
        match_chr: _,
        match_pos: _,
    } = association; // Exhaustiveness check of all fields

    match (dosage_0_weight, dosage_1_weight, dosage_2_weight) {
        (Some(dosage_0_weight), Some(dosage_1_weight), Some(dosage_2_weight)) => {
            // assert_eq!(None, dosage_0_weight);
            // assert_eq!(None, dosage_1_weight);
            // assert_eq!(None, dosage_2_weight);
            assert_eq!(None, is_interaction);
            assert_eq!(None, is_dominant);
            assert_eq!(None, is_recessive);
            assert_eq!(None, hr);
            // assert_eq!(None, or);
            Some(Effect::DosageSpecific {
                dosage_0_weight,
                dosage_1_weight,
                dosage_2_weight,
                effect_weight,
                or,
            })
        }
        (None, None, Some(_))
        | (None, Some(_), None)
        | (None, Some(_), Some(_))
        | (Some(_), None, None)
        | (Some(_), None, Some(_))
        | (Some(_), Some(_), None) => unreachable!(),
        (None, None, None) => None,
    }
}
