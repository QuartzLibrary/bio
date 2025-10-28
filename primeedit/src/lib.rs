pub mod design_spec;
pub mod edit;
pub mod editor;

use std::{collections::HashSet, hash::Hash, ops::Range};

use biocore::{
    dna::{DnaBase, DnaSequence},
    genome::{Contig, EditedContig},
    location::{ContigPosition, ContigRange, orientation::WithOrientation},
    mutation::SilentMutation,
};
use serde::{Deserialize, Serialize};
use utile::num::TryUsize;

use crate::{design_spec::DesignSpec, edit::Edit, editor::Editor};

pub type Pam<C> = WithOrientation<ContigRange<C>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Design<C> {
    pub edit: Edit<C>,
    pub editor: Editor,
    pub pam: Pam<C>,
    /// See [DesignSpec::force_5_prime_g].
    pub force_5_prime_g: bool,
    /// See [DesignSpec::primer_size_range].
    pub primer_size: u64,
    /// See [DesignSpec::rtt_template_homology_range].
    pub rtt_template_homology: u64,
    pub distruptions: Vec<SilentMutation<EditedContig<C>, DnaBase>>,
}

impl<C> Design<C>
where
    C: Contig + Clone,
{
    pub fn with_distruptions(
        self,
        distruptions: Vec<SilentMutation<EditedContig<C>, DnaBase>>,
    ) -> Self {
        Self {
            distruptions,
            ..self
        }
    }

    pub fn full_guide_sequence(&self) -> Option<DnaSequence> {
        let mut guide = self.spacer_sequence()?;
        guide.append(&mut self.editor.scaffold.clone());
        guide.append(&mut self.reverse_transcriptase_template()?);
        guide.append(&mut self.primer_sequence()?);
        Some(guide)
    }

    /// Checks whether the guide RNA has a poly-U tract.
    ///
    /// Note that this does not check the scaffold, since the default one does have poly-U tracts.
    /// See [DesignSpec::avoid_poly_u].
    pub fn has_poly_u(&self) -> Option<bool> {
        const PATTERN: [DnaBase; 4] = [DnaBase::T; 4];

        let spacer = self.spacer_sequence()?;
        let mut rest = self.reverse_transcriptase_template()?;
        rest.append(&mut self.primer_sequence()?);
        Some(spacer.contains(&PATTERN) || rest.contains(&PATTERN))
    }
    /// See [DesignSpec::avoid_rtt_cytosine].
    pub fn has_rtt_cytosine(&self) -> Option<bool> {
        let rtt_template = self.reverse_transcriptase_template()?;

        Some(rtt_template.first() == Some(&DnaBase::C))
    }
    /// See [DesignSpec::require_seed_or_pam_disruption].
    pub fn distrupts_pam(&self) -> Option<bool> {
        let editor = &self.editor;

        let rtt_template = self.reverse_transcriptase_template()?.reverse_complement();

        assert!(3 < rtt_template.len());

        // TODO: what if you get an insertion or deletion in the editable section of the seed?
        // Technically the pam might not be distrupted.

        let pam_size = editor.pam_size().usize_unwrap();
        let nick = editor.nick_distance.usize_unwrap();
        let new_pam_sequence = rtt_template[nick..(nick + pam_size)].encode();

        let regex = edit::cached_regex(&editor.pam_pattern);

        Some(!regex.is_match(&new_pam_sequence))
    }
    /// See [DesignSpec::require_seed_or_pam_disruption].
    pub fn distrupts_seed(&self) -> Option<bool> {
        let edit = &self.edit;
        let editor = &self.editor;

        let editable_seed_sequence =
            edit.get_original(edit.editable_seed(editor, self.pam.clone())?);

        let rtt_template = self.reverse_transcriptase_template()?.reverse_complement();

        assert!(editable_seed_sequence.len() < rtt_template.len());

        let nick = editor.nick_distance.usize_unwrap();

        Some(editable_seed_sequence[0..nick] != rtt_template[0..nick])
    }

    /// The location of the nick on the original sequence.
    pub fn nick(&self) -> Option<WithOrientation<ContigPosition<C>>> {
        self.edit.nick(&self.editor, self.pam.clone())
    }
    /// The location of the nick on the edited sequence.
    pub fn nick_in_edited(&self) -> Option<WithOrientation<ContigPosition<EditedContig<C>>>> {
        self.edit.nick_in_edited(&self.editor, self.pam.clone())
    }

    /// The spacer matches the sequence on the same side as the PAM, and will anneal to the opposite side.
    pub fn spacer(&self) -> Option<WithOrientation<ContigRange<C>>> {
        self.edit.spacer(&self.editor, self.pam.clone())
    }
    /// The CAS target is on the opposite strand as the PAM, it's what the spacer will anneal to.
    pub fn cas_target(&self) -> Option<WithOrientation<ContigRange<C>>> {
        self.edit.cas_target(&self.editor, self.pam.clone())
    }
    /// The sequence of the spacer, this includes forcing the first base to be a G if [Self::force_5_prime_g] is true.
    pub fn spacer_sequence(&self) -> Option<DnaSequence> {
        let mut spacer = self.edit.get_original(self.spacer()?);
        if self.force_5_prime_g {
            spacer[0] = DnaBase::G;
        }
        Some(spacer)
    }

    /// The section between the nick and the PAM.
    pub fn editable_seed(&self) -> Option<WithOrientation<ContigRange<C>>> {
        self.edit.editable_seed(&self.editor, self.pam.clone())
    }

    /// The PBS (primer binding site) target is the flap on the same side as the PAM,
    /// it's what will function as a jumping off point for the reverse transcription.
    pub fn primer_binding_site(&self) -> Option<WithOrientation<ContigRange<C>>> {
        self.edit
            .primer_binding_site(&self.editor, self.pam.clone(), self.primer_size)
    }
    /// The primer matches the sequence on the opposite side as the PAM.
    pub fn primer(&self) -> Option<WithOrientation<ContigRange<C>>> {
        self.edit
            .primer(&self.editor, self.pam.clone(), self.primer_size)
    }
    /// The sequence of the primer section of the guide RNA.
    pub fn primer_sequence(&self) -> Option<DnaSequence> {
        Some(self.edit.get_original(self.primer()?))
    }

    pub fn editable_range_in_edit(&self) -> Option<WithOrientation<ContigRange<EditedContig<C>>>> {
        let edited_contig = self.edit.edited_contig();
        let range = self.edit.editable_range(&self.editor, self.pam.clone())?;
        Some(edited_contig.clone().liftover_range(range).unwrap())
    }
    pub fn non_editable_range_in_edit(
        &self,
    ) -> Option<WithOrientation<ContigRange<EditedContig<C>>>> {
        let edited_contig = self.edit.edited_contig();
        let range = self
            .edit
            .non_editable_range(&self.editor, self.pam.clone())?;
        Some(edited_contig.clone().liftover_range(range).unwrap())
    }

    /// Returns the sequence of the reverse transcriptase template.
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

        let edited_contig = edit.edited_contig();
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

        {
            let full_edited_range_in_original = edit.edit_range_in_original().into_forward().v;
            let full_edited_range = edit.edit_range_in_edited().into_forward().v;
            let original = edit.original();
            assert_eq!(
                original[..full_edited_range_in_original.clone().into_start()],
                edited[..full_edited_range.clone().into_start()]
            );
            assert_eq!(
                original[full_edited_range_in_original.clone().into_end()..],
                edited[full_edited_range.clone().into_end()..]
            );
            let original = &original[full_edited_range_in_original];
            let edited = &edited[full_edited_range];
            if self.edit.has_any_effect() {
                assert_ne!(original, edited);
                if let (Some(original), Some(edited)) = (original.first(), edited.first()) {
                    assert_ne!(original, edited);
                }
                if let (Some(original), Some(edited)) = (original.last(), edited.last()) {
                    assert_ne!(original, edited);
                }
            }
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
    pub fn full_edited_range_in_edited(&self) -> WithOrientation<ContigRange<EditedContig<C>>> {
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
        let edited_contig = edit.edited_contig();

        assert_eq!(edited_contig, edit.edit_range_in_edited().v.contig);

        let Range { mut start, mut end } = edit
            .edit_range_in_edited()
            .into_orientation(orientation)
            .v
            .at;

        for m in distruptions {
            assert_eq!(edited_contig, m.at.v.contig);
            let Range {
                start: m_start,
                end: m_end,
            } = m.affected_range().into_orientation(orientation).v.at;
            start = start.min(m_start);
            end = end.max(m_end);
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
impl<C> Design<C>
where
    C: Contig + Clone + Hash + PartialEq,
{
    pub fn is_valid(&self) -> bool {
        let distruptions: HashSet<_> = self
            .edit
            .edited_silent_mutations()
            .into_iter()
            .flatten()
            .collect();

        self.edit.pams(&self.editor).contains(&self.pam)
            && self.distruptions.iter().all(|m| distruptions.contains(m))
            && self.is_in_range()
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
            && if let Some(force_5_prime_g) = force_5_prime_g {
                let equal = *force_5_prime_g == self.force_5_prime_g;
                let last_5_prime_is_g = self
                    .unforced_spacer_sequence()
                    .map(|s| s[0])
                    .is_none_or(|b| b == DnaBase::G);
                equal || last_5_prime_is_g
            } else {
                true
            }
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
    fn is_in_range(&self) -> bool {
        self.spacer().is_some()
            && self.primer_binding_site().is_some()
            && self.reverse_transcriptase_template().is_some()
    }
}

impl<C> Design<C>
where
    C: Contig + Clone + Hash + PartialEq,
{
    pub fn assert_valid(&self) {
        assert!(self.edit.pams(&self.editor).contains(&self.pam));

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
    pub fn assert_compliant(&self, spec: &DesignSpec) {
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
        assert!(editor == &self.editor);
        if let Some(force_5_prime_g) = force_5_prime_g {
            let equal = *force_5_prime_g == self.force_5_prime_g;
            let last_5_prime_is_g = self
                .unforced_spacer_sequence()
                .map(|s| s[0])
                .is_none_or(|b| b == DnaBase::G);
            assert!(equal || last_5_prime_is_g);
        }
        assert!(primer_size_range.contains(&self.primer_size));
        assert!(rtt_template_homology_range.contains(&self.rtt_template_homology));
        if *avoid_poly_u {
            assert!(!self.has_poly_u().expect("sizing"));
        }
        if *avoid_rtt_cytosine {
            assert!(!self.has_rtt_cytosine().expect("sizing"));
        }
        if *require_seed_or_pam_disruption {
            assert!(
                self.distrupts_seed().expect("sizing") || self.distrupts_pam().expect("sizing")
            );
        }
        if let Some(minimum_edit_size) = minimum_edit_size {
            assert!(*minimum_edit_size <= self.full_edited_range_in_edited().v.len());
        }
    }
    fn assert_in_range(&self) {
        assert!(self.spacer().is_some());
        assert!(self.primer_binding_site().is_some());
        assert!(self.reverse_transcriptase_template().is_some());
    }
}

impl<C> Design<C>
where
    C: Contig + Clone + Hash + PartialEq,
{
    fn unforced_spacer_sequence(&self) -> Option<DnaSequence> {
        Some(self.edit.get_original(self.spacer()?))
    }
    /// Any mutation that affects the bases surrounding the edit, excluding the seed and pam.
    // TODO: right now it just checks the selected range is affected, not that a mutation doesn't spill outside of the cap.
    fn _mutation_in_edit_is_mmr_evading(
        &self,
        m: &SilentMutation<EditedContig<C>, DnaBase>,
        cap: u64,
    ) -> bool {
        let edited_contig = self.edit.edited_contig();
        let affected_positions: HashSet<_> = m.affected_positions().collect();
        self.edit
            ._mmr_evading_ranges(self.pam.clone(), cap)
            .into_iter()
            .flatten()
            .map(|r| {
                edited_contig
                    .clone()
                    .liftover_range(r)
                    .expect("mmr evading does not overlap edit")
            })
            .flat_map(|p| p.iter_positions())
            .any(|p| affected_positions.contains(&p))
    }
}
