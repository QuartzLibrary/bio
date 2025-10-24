use std::{iter, ops::Range};

use biocore::{
    dna::DnaBase,
    genome::{Contig, EditedContig},
    location::orientation::WithOrientation,
};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use utile::any::AnyMap;

use crate::{ContigRange, Design, Editor, SilentMutation, edit::Edit};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct DesignSpec {
    pub editor: Editor,
    /// "Importantly, if the epegRNA will be expressed from a plasmid via the U6 RNA polymerase III promoter,
    /// a 5′ G at the start of the spacer is necessary to initiate transcription efficiently and
    /// should be incorporated into the epegRNA design."
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    pub force_5_prime_g: Option<bool>,
    /// "In our experiences, optimal PBS lengths have ranged from 8 to 15 nt"
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    pub primer_size_range: Range<u64>,
    /// "the optimal RTT range is even larger (10 to 74 nt)"
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    pub rtt_template_homology_range: Range<u64>,
    /// "For epegRNAs expressed from a plasmid using the U6 RNA polymerase III promoter, four or more consecutive
    /// uridines in the pegRNA sequence may act as a transcriptional terminator and prematurely truncate the epegRNA 66.
    /// Therefore, the sequences of the spacer, PBS, and RTT should avoid such poly(U) tracts if possible."
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    pub avoid_poly_u: bool,
    /// "Additionally, we (but not others) have observed that beginning the RTT sequence with a cytosine lowers editing,
    /// likely because it disturbs the structure of the epegRNA scaffold. Therefore, we recommend designing the 3′ extension
    /// to not begin with cytosine and omitting designs that would do so when screening for optimal RTTs."
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    /// "Importantly, RT templates that place a C adjacent to the 3’ hairpin of the sgRNA scaffold generally resulted
    /// in lower editing efficiency. We speculate that a C as the first nucleotide of the
    /// 3’ extension can disrupt guide RNA structure by pairing with G81, which normally forms a pi stack with Y1356
    /// in Cas9 and a non-canonical base pair with sgRNA A6828. Since many RT template lengths support prime editing,
    /// we recommend designing pegRNAs such that the first base of the 3’ extension is not C."
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC6907074/
    pub avoid_rtt_cytosine: bool,
    /// "mutations that disrupt either the PAM (positions +5–6) or the seed region (positions +1–3) of the target site.
    /// PAM or seed-disrupting edits partially prevent Cas9 from re-binding and re-nicking the target strand,
    /// which otherwise could result in indels or the reversion of a desired edit back to the wild-type sequence15.
    /// To include PAM or seed-disrupting mutations, simply encode them in the RTT of the epegRNA along with the original target edit.
    /// PAM-disrupting and seed-disrupting mutations are almost always beneficial, and we recommend including them if possible."
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    pub require_seed_or_pam_disruption: bool,
    /// "MMR-evading target-adjacent mutations. Because the inclusion of additional mutations adjacent to the target
    /// mutation results in more significant helix distortion, these regions are less likely to be recognized by
    /// cellular MMR proteins. This strategy is particularly useful for desired edits that are point mutations and
    /// insertions and deletions of fewer than 13 nt. To include MMR-evading mutations, encode them in the RTT of
    /// the epegRNA along with the desired edit (Fig. 3). Silent mismatches (particularly C•C mismatches), within 5 nt
    /// of the desired edit are typically the most beneficial. Notably, the effects of MMR-evading mutations are less
    /// consistent than those of PAM-disrupting mutations, and certain mismatch types are more effective than others."
    /// – https://pmc.ncbi.nlm.nih.gov/articles/PMC9799714/
    ///
    /// This is the minimum size of the edit (the distance between the first and last edited bases).
    pub minimum_edit_size: Option<u64>,
}
impl Default for DesignSpec {
    fn default() -> Self {
        Self {
            editor: Editor::sp_cas9(),
            force_5_prime_g: Some(true),
            primer_size_range: 8..15,
            rtt_template_homology_range: 10..30, // 10..74,
            avoid_poly_u: true,
            avoid_rtt_cytosine: true,
            require_seed_or_pam_disruption: false,
            minimum_edit_size: None,
        }
    }
}
impl DesignSpec {
    /// Returns all valid designs for the given edit and PAM.
    ///
    /// If you want to include distruptions, use [Self::designs_with_distruptions].
    pub fn designs<C>(
        self,
        edit: Edit<C>,
        pam: WithOrientation<ContigRange<C>>,
    ) -> Option<impl Iterator<Item = Design<C>> + use<C>>
    where
        C: Contig + Clone + Hash,
    {
        self.designs_with_distruptions(edit.clone(), pam.clone(), vec![])
    }
    /// Returns all valid designs for the given edit and PAM, with or without distruptions.
    ///
    /// If you don't want to include distruptions, use [Self::designs].
    // TODO: better name, and maybe make it combinatorial?
    pub fn designs_with_and_without_distruptions<C>(
        self,
        edit: Edit<C>,
        pam: WithOrientation<ContigRange<C>>,
        distruptions: Vec<SilentMutation<EditedContig<C>, DnaBase>>,
    ) -> Option<impl Iterator<Item = Design<C>> + use<C>>
    where
        C: Contig + Clone + Hash,
    {
        let any_distruptions = !distruptions.is_empty();
        Some(
            self.clone()
                .designs_with_distruptions(edit.clone(), pam.clone(), distruptions)?
                .chain(
                    any_distruptions
                        .then(|| self.designs_with_distruptions(edit, pam, vec![]).unwrap())
                        .into_iter()
                        .flatten(),
                ),
        )
    }
    fn designs_with_distruptions<C>(
        self,
        edit: Edit<C>,
        pam: WithOrientation<ContigRange<C>>,
        distruptions: Vec<SilentMutation<EditedContig<C>, DnaBase>>,
    ) -> Option<impl Iterator<Item = Design<C>> + use<C>>
    where
        C: Contig + Clone + Hash,
    {
        let Self {
            editor,
            force_5_prime_g,
            primer_size_range,
            rtt_template_homology_range,
            avoid_poly_u,
            avoid_rtt_cytosine,
            require_seed_or_pam_disruption,
            minimum_edit_size,
        } = self.clone();

        if !edit.pams(&self.editor).contains(&pam) {
            return None;
        }

        [pam]
            .into_iter()
            .zip(iter::repeat(editor))
            .zip(iter::repeat(edit.clone()))
            .zip(iter::repeat(distruptions.clone()))
            .map(|(((pam, editor), edit), distruptions)| Design {
                edit,
                editor,
                pam,
                force_5_prime_g: false,
                primer_size: 10,
                rtt_template_homology: 14,
                distruptions,
            })
            // First sizing
            .flat_map(move |design| {
                iter::repeat(design).zip(primer_size_range.clone()).map(
                    |(mut design, primer_size)| {
                        design.primer_size = primer_size;
                        design
                    },
                )
            })
            .flat_map(move |design| {
                iter::repeat(design)
                    .zip(rtt_template_homology_range.clone())
                    .map(|(mut design, rtt_template_homology)| {
                        design.rtt_template_homology = rtt_template_homology;
                        design
                    })
            })
            // Checks sizing, so we don't have to worry about that.
            .filter(|design| design.is_in_range())
            .flat_map(move |design| {
                let spacer_range = design.spacer().expect("sizing");
                let first_spacer_base = design.edit.get_original(spacer_range)[0];

                let forced = {
                    let mut design = design.clone();
                    design.force_5_prime_g = true;
                    design
                };

                match force_5_prime_g {
                    _ if first_spacer_base == DnaBase::G => [Some(design), None],

                    Some(true) => [Some(forced), None],
                    Some(false) => [Some(design), None],
                    None => [Some(design), Some(forced)],
                }
                .into_iter()
                .flatten()
            })
            .filter(move |design| {
                if avoid_poly_u {
                    !design.has_poly_u().expect("sizing")
                } else {
                    true
                }
            })
            .filter(move |design| {
                if avoid_rtt_cytosine {
                    !design.has_rtt_cytosine().expect("sizing")
                } else {
                    true
                }
            })
            .filter(move |design| {
                if require_seed_or_pam_disruption {
                    design.distrupts_seed().expect("sizing")
                        || design.distrupts_pam().expect("sizing")
                } else {
                    true
                }
            })
            .filter(move |design| {
                minimum_edit_size
                    .is_none_or(|min| min <= design.full_edited_range_in_edited().v.len())
            })
            .inspect(|design| design.assert_valid())
            .any_map(Some)
    }
}
