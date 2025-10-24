pub mod design_spec;
pub mod edit;
pub mod editor;

use std::{collections::HashSet, ops::Range};

use biocore::{
    dna::{DnaBase, DnaSequence},
    genome::ArcContig,
    location::orientation::WithOrientation,
};
use serde::{Deserialize, Serialize};
use utile::num::TryUsize;

use crate::{design_spec::DesignSpec, edit::Edit, editor::Editor};

pub type ContigPosition = biocore::location::ContigPosition<ArcContig>;
pub type ContigRange = biocore::location::ContigRange<ArcContig>;

pub type SilentMutation = biocore::mutation::SilentMutation<ArcContig, DnaBase>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Design {
    pub edit: Edit,
    pub editor: Editor,
    pub pam: WithOrientation<ContigRange>,
    pub force_5_prime_g: bool,
    pub primer_size: u64,
    pub rtt_template_homology: u64,
    pub distruptions: Vec<SilentMutation>,
}

impl Design {
    fn assert_valid(&self) {
        assert!(self.pams().contains(&self.pam));

        let distruptions: HashSet<_> = self
            .edit
            .edited_silent_mutations()
            .into_iter()
            .flatten()
            .collect();

        for m in &self.distruptions {
            assert!(distruptions.contains(m));
        }

        assert!(self.spacer().is_some());
        assert!(self.reverse_transcriptase_template().is_some());
        self.assert_in_range();

        assert!(self.is_valid());
    }
    fn assert_in_range(&self) {
        assert!(self.spacer().is_some());
        assert!(self.primer_binding_site().is_some());
        assert!(self.reverse_transcriptase_template().is_some());
    }

    pub fn is_compliant(&self, spec: &DesignSpec) -> bool {
        let DesignSpec {
            editor,
            force_5_prime_g,
            primer_size_range,
            rtt_template_homology_range,
            avoid_poly_u,
            avoid_rtt_cytosine,
            require_seed_or_pam_disruption,
            minimum_edit_size,
        } = spec;
        {
            let Self {
                edit: _,
                editor: _,
                pam: _,
                force_5_prime_g: _,
                primer_size: _,
                rtt_template_homology: _,
                distruptions: _,
            } = self; // Exhaustiveness check.
        }
        self.is_valid()
            && editor == &self.editor
            && force_5_prime_g.is_none_or(|f| f == self.force_5_prime_g)
            && primer_size_range.contains(&self.primer_size)
            && rtt_template_homology_range.contains(&self.rtt_template_homology)
            && if *avoid_poly_u {
                !self.has_poly_u().expect("sizing")
            } else {
                true
            }
            && if *avoid_rtt_cytosine {
                !self.has_rtt_cytosine().expect("sizing")
            } else {
                true
            }
            && if *require_seed_or_pam_disruption {
                self.distrupts_seed().expect("sizing") || self.distrupts_pam().expect("sizing")
            } else {
                true
            }
            && minimum_edit_size.is_none_or(|min| min <= self.full_edited_range_in_edited().v.len())
    }

    pub fn is_valid(&self) -> bool {
        let distruptions: HashSet<_> = self
            .edit
            .edited_silent_mutations()
            .into_iter()
            .flatten()
            .collect();

        self.pams().contains(&self.pam)
            && self.distruptions.iter().all(|m| distruptions.contains(m))
            && self.is_in_range()
    }
    pub fn with_distruptions(self, distruptions: Vec<SilentMutation>) -> Self {
        Self {
            distruptions,
            ..self
        }
    }
    fn is_in_range(&self) -> bool {
        self.spacer().is_some()
            && self.primer_binding_site().is_some()
            && self.reverse_transcriptase_template().is_some()
    }

    fn mutation_in_edit_affects_pam(&self, m: &SilentMutation) -> bool {
        let edited_contig = self.edit.edited_contig();
        let affected_positions: HashSet<_> = m.affected_positions().collect();
        self.pam
            .iter_positions()
            .filter_map(move |p| edited_contig.clone().liftover(p))
            .map(|p| p.map_value(|p| p.map_contig(ArcContig::from_contig)))
            .any(|p| affected_positions.contains(&p))
    }
    fn mutation_in_edit_affects_seed(&self, m: &SilentMutation) -> bool {
        let edited_contig = self.edit.edited_contig();
        let affected_positions: HashSet<_> = m.affected_positions().collect();
        self.editable_seed()
            .into_iter()
            .flat_map(|editable_seed| editable_seed.iter_positions())
            .filter_map(move |p| edited_contig.clone().liftover(p))
            .map(|p| p.map_value(|p| p.map_contig(ArcContig::from_contig)))
            .any(|p| affected_positions.contains(&p))
    }
    /// Any mutation that affects the bases surrounding the edit, excluding the seed and pam.
    // TODO: right now it just checks the selected range is affected, not that a mutation doesn't spill outside of the cap.
    fn _mutation_in_edit_is_mmr_evading(&self, m: &SilentMutation, cap: u64) -> bool {
        let edited_contig = self.edit.edited_contig();
        let affected_positions: HashSet<_> = m.affected_positions().collect();
        self.edit
            .mmr_evading_ranges(self.pam.clone(), cap)
            .into_iter()
            .flatten()
            .map(|r| {
                edited_contig
                    .clone()
                    .liftover_range(r)
                    .expect("mmr evading does not overlap edit")
            })
            .map(|p| p.map_value(|p| p.map_contig(ArcContig::from_contig)))
            .flat_map(|p| p.iter_positions())
            .any(|p| affected_positions.contains(&p))
    }

    pub fn editable_range_in_edit(&self) -> Option<WithOrientation<ContigRange>> {
        let edited_contig = self.edit.edited_contig();
        let range = self.edit.editable_range(&self.editor, self.pam.clone())?;
        Some(
            edited_contig
                .clone()
                .liftover_range(range)
                .unwrap()
                .map_value(|v| v.map_contig(ArcContig::from_contig)),
        )
    }
    pub fn non_editable_range_in_edit(&self) -> Option<WithOrientation<ContigRange>> {
        let edited_contig = self.edit.edited_contig();
        let range = self
            .edit
            .non_editable_range(&self.editor, self.pam.clone())?;
        Some(
            edited_contig
                .clone()
                .liftover_range(range)
                .unwrap()
                .map_value(|v| v.map_contig(ArcContig::from_contig)),
        )
    }

    pub fn has_poly_u(&self) -> Option<bool> {
        const PATTERN: [DnaBase; 4] = [DnaBase::T; 4];

        let spacer = self.spacer_sequence()?;
        let mut rest = self.reverse_transcriptase_template()?;
        rest.append(&mut self.primer_sequence()?);
        Some(spacer.contains(&PATTERN) || rest.contains(&PATTERN))
    }
    pub fn has_rtt_cytosine(&self) -> Option<bool> {
        let rtt_template = self.reverse_transcriptase_template()?;

        Some(rtt_template.last() == Some(&DnaBase::C))
    }
    pub fn distrupts_pam(&self) -> Option<bool> {
        let edit = &self.edit;
        let editor = &self.editor;

        let pam_sequence = edit.get_original(self.pam.clone());
        let rtt_template = self.reverse_transcriptase_template()?;

        assert_eq!(3, pam_sequence.len());
        assert!(3 < rtt_template.len());

        let nick = editor.nick_distance.usize_unwrap();

        // TODO: what if you get an insertion or deletion in the editable section of the seed?
        // Technically the pam might not be distrupted.

        Some(
            pam_sequence[1..3] != *rtt_template[(nick + 1)..(nick + 3)].reverse_complement()
                || self
                    .distruptions
                    .iter()
                    .any(|m| self.mutation_in_edit_affects_pam(m)),
        )
    }
    pub fn distrupts_seed(&self) -> Option<bool> {
        let edit = &self.edit;
        let editor = &self.editor;

        let editable_seed_sequence =
            edit.get_original(edit.editable_seed(editor, self.pam.clone())?);

        let rtt_template = self.reverse_transcriptase_template()?;

        assert!(editable_seed_sequence.len() < rtt_template.len());

        let nick = editor.nick_distance.usize_unwrap();

        Some(
            editable_seed_sequence[0..nick] != *rtt_template[0..nick].reverse_complement()
                || self
                    .distruptions
                    .iter()
                    .any(|m| self.mutation_in_edit_affects_seed(m)),
        )
    }
}

impl Design {
    pub fn full_guide_sequence(&self) -> Option<DnaSequence> {
        let mut guide = self.spacer_sequence()?;
        guide.append(&mut self.editor.scaffold.clone());
        guide.append(&mut self.reverse_transcriptase_template()?);
        guide.append(&mut self.primer_sequence()?);
        Some(guide)
    }

    pub fn pams(&self) -> Vec<WithOrientation<ContigRange>> {
        self.edit.pams(&self.editor)
    }

    pub fn nick(&self) -> Option<WithOrientation<ContigPosition>> {
        self.edit.nick(&self.editor, self.pam.clone())
    }
    pub fn nick_in_edited(&self) -> Option<WithOrientation<ContigPosition>> {
        self.edit.nick_in_edited(&self.editor, self.pam.clone())
    }

    /// The spacer matches the sequence on the same side as the PAM, and will anneal to the opposite side.
    pub fn spacer(&self) -> Option<WithOrientation<ContigRange>> {
        self.edit.spacer(&self.editor, self.pam.clone())
    }
    /// The CAS target is on the opposite strand as the PAM, it's what the spacer will anneal to.
    pub fn cas_target(&self) -> Option<WithOrientation<ContigRange>> {
        self.edit.cas_target(&self.editor, self.pam.clone())
    }
    pub fn spacer_sequence(&self) -> Option<DnaSequence> {
        let mut spacer = self.edit.get_original(self.spacer()?);
        if self.force_5_prime_g {
            spacer[0] = DnaBase::G;
        }
        Some(spacer)
    }

    /// The section between the nick and the PAM.
    pub fn editable_seed(&self) -> Option<WithOrientation<ContigRange>> {
        self.edit.editable_seed(&self.editor, self.pam.clone())
    }

    pub fn primer_binding_site(&self) -> Option<WithOrientation<ContigRange>> {
        self.edit
            .primer_binding_site(&self.editor, self.pam.clone(), self.primer_size)
    }
    /// The primer matches the sequence on the opposite side as the PAM.
    pub fn primer(&self) -> Option<WithOrientation<ContigRange>> {
        self.edit
            .primer(&self.editor, self.pam.clone(), self.primer_size)
    }
    pub fn primer_sequence(&self) -> Option<DnaSequence> {
        Some(self.edit.get_original(self.primer()?))
    }

    /// Returns the sequence of the reverse transcriptase template and the range it covers in the original sequence.
    pub fn reverse_transcriptase_template(&self) -> Option<DnaSequence> {
        let Self {
            edit,
            editor,
            pam,
            force_5_prime_g: _,
            primer_size: _,
            rtt_template_homology,
            distruptions,
        } = self;

        let edited_contig = ArcContig::from_contig(edit.edited_contig());
        let orientation = pam.orientation;

        let nick = edit.nick_in_edited(editor, pam.clone())?;
        assert_eq!(orientation, nick.orientation);
        assert_eq!(edited_contig, nick.v.contig);

        let full_edited_range = self.full_edited_range_in_edited();
        assert_eq!(orientation, full_edited_range.orientation);
        assert_eq!(edited_contig, full_edited_range.v.contig);

        let rtt_range = WithOrientation {
            orientation,
            v: ContigRange {
                contig: edited_contig,
                at: nick.v.at..(full_edited_range.v.at.end + rtt_template_homology),
            },
        };

        let mut edited = edit.edited();
        for m in distruptions {
            m.apply(&mut edited);
        }
        if rtt_range.orientation.is_reverse() {
            edited = edited.reverse_complement();
        }

        Some(
            edited
                .get_range(rtt_range.v.usize_range())?
                .reverse_complement(),
        )
    }
    /// The full edited range, including silent mutations.
    pub fn full_edited_range_in_edited(&self) -> WithOrientation<ContigRange> {
        let Self {
            edit,
            editor: _,
            pam,
            force_5_prime_g: _,
            primer_size: _,
            rtt_template_homology: _,
            distruptions,
        } = self;
        let orientation = pam.orientation;
        let edited_contig = ArcContig::from_contig(edit.edited_contig());

        assert_eq!(edited_contig, edit.edit_range_in_edited().v.contig);

        let Range { mut start, mut end } = edit
            .edit_range_in_edited()
            .into_orientation(orientation)
            .v
            .at;

        for m in distruptions {
            assert_eq!(edited_contig, m.at_range().v.contig);
            let Range {
                start: m_start,
                end: m_end,
            } = m.at_range().into_orientation(orientation).v.at;
            if m_start < start {
                start = m_start;
            }
            if end < m_end {
                end = m_end;
            }
        }

        WithOrientation {
            orientation,
            v: ContigRange {
                contig: edited_contig,
                at: start..end,
            },
        }
    }
}
