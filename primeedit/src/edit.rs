use std::{
    collections::BTreeMap,
    ops::Range,
    sync::{Arc, LazyLock, Mutex},
};

use biocore::{
    dna::{DnaBase, DnaSequence, IupacDnaBase, IupacDnaSequenceSlice},
    genome::{ArcContig, Contig, ContigRef, EditedContig},
    location::{
        ContigPosition, ContigRange,
        orientation::{SequenceOrientation, WithOrientation},
    },
    mutation::SilentMutation,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use utile::{
    num::{TryI64, TryU64},
    range::RangeLen,
    regex_ext::find_iter_overlapping,
};

use crate::{Pam, editor::Editor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Edit<C> {
    contig: C,

    start: DnaSequence,
    original: DnaSequence,
    edited: DnaSequence,
    end: DnaSequence,

    /// On the sense/RNA-like strand (as opposed to the antisense/template strand).
    translation_frame_start: Option<WithOrientation<ContigPosition<C>>>,
}
impl Edit<ArcContig> {
    pub fn parse(input: &str) -> Option<Self> {
        let (start, rest) = input.split_once('(')?;
        let (mut original, rest) = rest.split_once('/')?;
        let (mut edited, end) = rest.split_once(')')?;

        let mut start = start.to_owned();
        let mut end = end.to_owned();

        while let Some(o) = original.chars().next()
            && let Some(e) = edited.chars().next()
            && o == e
        {
            original = original.strip_prefix(o).unwrap();
            edited = edited.strip_prefix(e).unwrap();
            start = format!("{start}{o}");
        }

        while let Some(o) = original.chars().next_back()
            && let Some(e) = edited.chars().next_back()
            && o == e
        {
            original = original.strip_suffix(o).unwrap();
            edited = edited.strip_suffix(e).unwrap();
            end = format!("{e}{end}");
        }

        let start: DnaSequence = start.parse().ok()?;
        let original: DnaSequence = original.parse().ok()?;
        let edited: DnaSequence = edited.parse().ok()?;
        let end: DnaSequence = end.parse().ok()?;

        let contig = ArcContig::from_contig(ContigRef::new(
            "original",
            (start.len() + original.len() + end.len()).u64_unwrap(),
        ));

        Some(Self {
            contig,

            start,
            original,
            edited,
            end,

            translation_frame_start: None,
        })
    }
    pub fn with_translation_frame_start(
        mut self,
        translation_frame_start: WithOrientation<ContigPosition<ArcContig>>,
    ) -> Self {
        self.translation_frame_start = Some(translation_frame_start);
        self
    }
}
impl<C> Edit<C>
where
    C: Contig + Clone,
{
    pub fn has_any_effect(&self) -> bool {
        self.original != self.edited // Both could be empty
    }
    pub fn original_len(&self) -> usize {
        self.start.len() + self.original.len() + self.end.len()
    }
    pub fn original_contig(&self) -> C {
        self.contig.clone()
    }
    pub fn original(&self) -> DnaSequence {
        let Self {
            contig: _,

            start,
            original,
            edited: _,
            end,
            translation_frame_start: _,
        } = self;
        format!("{start}{original}{end}").parse().unwrap()
    }
    pub fn edit_range_in_original(&self) -> WithOrientation<ContigRange<C>> {
        let Self {
            contig: _,
            start,
            original,
            edited: _,
            end: _,
            translation_frame_start: _,
        } = self;
        let range = start.len()..start.len() + original.len();
        let range = u64::try_from(range.start).unwrap()..u64::try_from(range.end).unwrap();
        WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigRange {
                contig: self.original_contig(),
                at: range,
            },
        }
    }
    #[track_caller]
    pub fn get_original(&self, range: WithOrientation<ContigRange<C>>) -> DnaSequence {
        assert_eq!(self.original_contig(), range.v.contig);
        match range.orientation {
            SequenceOrientation::Forward => self.original()[range.v].to_owned(),
            SequenceOrientation::Reverse => {
                self.original().reverse_complement()[range.v].to_owned()
            }
        }
    }

    pub fn edited_len(&self) -> usize {
        self.start.len() + self.edited.len() + self.end.len()
    }
    pub fn edited_contig(&self) -> EditedContig<C> {
        let original_contig = self.original_contig();
        let remove = self.edit_range_in_original().v;
        let insert = self.raw_edit_range_in_edited().range_len();
        EditedContig::new(original_contig, remove, insert)
    }
    pub fn edited(&self) -> DnaSequence {
        let Self {
            contig: _,
            start,
            original: _,
            edited,
            end,
            translation_frame_start: _,
        } = self;
        format!("{start}{edited}{end}").parse().unwrap()
    }
    pub fn edit_range_in_edited(&self) -> WithOrientation<ContigRange<EditedContig<C>>> {
        let range = self.raw_edit_range_in_edited();
        WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigRange {
                contig: self.edited_contig(),
                at: range,
            },
        }
    }
    fn raw_edit_range_in_edited(&self) -> Range<u64> {
        let Self {
            contig: _,
            start,
            original: _,
            edited,
            end: _,
            translation_frame_start: _,
        } = self;
        let range = start.len()..start.len() + edited.len();
        range.start.u64_unwrap()..range.end.u64_unwrap()
    }
    pub fn get_edited(&self, range: WithOrientation<ContigRange<EditedContig<C>>>) -> DnaSequence {
        assert_eq!(self.edited_contig(), range.v.contig);
        match range.orientation {
            SequenceOrientation::Forward => self.edited()[range.v].to_owned(),
            SequenceOrientation::Reverse => self.edited().reverse_complement()[range.v].to_owned(),
        }
    }

    pub fn diff_edit_range(&self) -> impl Iterator<Item = diff::Result<()>> {
        diff::slice(&self.original, &self.edited)
            .into_iter()
            .map(|v| match v {
                diff::Result::Left(_) => diff::Result::Left(()),
                diff::Result::Both(_, _) => diff::Result::Both((), ()),
                diff::Result::Right(_) => diff::Result::Right(()),
            })
    }

    /// Returns the PAM locations in the original sequence, all ranges are 5' → 3' (and annotated with the orientation).
    pub fn pams(&self, editor: &Editor) -> Vec<Pam<C>> {
        let Editor {
            pam_pattern,
            nick_distance,
            spacer_size: _,
            scaffold: _,
        } = editor;
        let pam_size = editor.pam_size();
        let nick_distance = *nick_distance;

        let original = self.original();

        let edit_range = self.edit_range_in_original();

        let mut forward = edit_range.clone().into_start().v;
        let mut reverse = edit_range.flip_orientation().into_start().v;

        // We can go slightly past the edit range because the nick is before the PAM.
        forward.saturating_add_assign(nick_distance);
        reverse.saturating_add_assign(nick_distance);
        // And we include the PAM size so that the regex can find it even if at the end.
        forward.saturating_add_assign(pam_size);
        reverse.saturating_add_assign(pam_size);

        let regex = cached_regex(pam_pattern);

        [].into_iter()
            .chain(
                find_iter_overlapping(&regex, &original[..forward].encode())
                    .map(|v| (v, SequenceOrientation::Forward)),
            )
            .chain(
                find_iter_overlapping(&regex, &original.reverse_complement()[..reverse].encode())
                    .map(|v| (v, SequenceOrientation::Reverse)),
            )
            .map(|(v, orientation)| {
                assert_eq!(pam_size, v.end.strict_sub(v.start));
                WithOrientation {
                    orientation,
                    v: ContigRange {
                        contig: self.original_contig(),
                        at: v.start..v.end,
                    },
                }
            })
            .collect()
    }
    /// Returns the PAM closest to the edit range, if any is present.
    pub fn closest_pam(&self, editor: &Editor) -> Option<Pam<C>> {
        let edit_start = self.edit_range_in_original().into_start();
        let edit_start = edit_start.as_ref_contig();
        self.pams(editor)
            .iter()
            .min_by_key(move |pam| {
                let edit_start = edit_start.into_orientation(pam.orientation).v.at;

                edit_start.i64_unwrap() - pam.v.at.start.i64_unwrap()
            })
            .cloned()
    }
    /// Returns the distance from the start of the pam to the start of the edit.
    ///
    /// Since the edit can cover the pam, the distance can be negative.
    pub fn distance_from_edit(&self, pam: Pam<C>) -> i64 {
        let edit_start = self
            .edit_range_in_original()
            .into_orientation(pam.orientation)
            .v
            .at
            .start
            .i64_unwrap();

        edit_start - pam.v.at.start.i64_unwrap()
    }

    /// The nick is on the same strand as the PAM, the side the spacer anneals to.
    pub fn nick(&self, editor: &Editor, pam: Pam<C>) -> Option<WithOrientation<ContigPosition<C>>> {
        Some(WithOrientation {
            orientation: pam.orientation,
            v: pam.into_start().v.checked_sub(editor.nick_distance)?,
        })
    }
    /// The nick in the edited contig, note that since the edit is always on the 3' side of the nick,
    /// the coordinates of the nick in the original and edited contigs are the same.
    pub fn nick_in_edited(
        &self,
        editor: &Editor,
        pam: Pam<C>,
    ) -> Option<WithOrientation<ContigPosition<EditedContig<C>>>> {
        Some(
            self.nick(editor, pam)?
                .map_value(|p| p.map_contig(|_| self.edited_contig())),
        )
    }
    /// The spacer matches the sequence on the same side as the PAM, and will anneal to the opposite side.
    pub fn spacer(&self, editor: &Editor, pam: Pam<C>) -> Option<WithOrientation<ContigRange<C>>> {
        let spacer_size = editor.spacer_size;
        if pam.v.at.start < spacer_size {
            return None;
        }
        let start = pam.v.at.start;
        Some(WithOrientation {
            orientation: pam.orientation,
            v: ContigRange {
                contig: pam.v.contig,
                at: start.checked_sub(spacer_size)?..start,
            },
        })
    }
    /// The CAS target is on the opposite strand as the PAM, it's what the spacer will anneal to.
    pub fn cas_target(
        &self,
        editor: &Editor,
        pam: Pam<C>,
    ) -> Option<WithOrientation<ContigRange<C>>> {
        Some(self.spacer(editor, pam)?.flip_orientation())
    }
    /// The section between the nick and the PAM.
    pub fn editable_seed(
        &self,
        editor: &Editor,
        pam: Pam<C>,
    ) -> Option<WithOrientation<ContigRange<C>>> {
        let start = pam.v.at.start;
        Some(WithOrientation {
            orientation: pam.orientation,
            v: ContigRange {
                contig: pam.v.contig,
                at: start.checked_sub(editor.nick_distance)?..start,
            },
        })
    }
    /// The entire range that is editable, from the nick onwards.
    pub fn editable_range(
        &self,
        editor: &Editor,
        pam: Pam<C>,
    ) -> Option<WithOrientation<ContigRange<C>>> {
        let start = pam.v.at.start;
        Some(WithOrientation {
            orientation: pam.orientation,
            v: ContigRange {
                contig: pam.v.contig,
                at: start.checked_sub(editor.nick_distance)?..self.original_len().u64_unwrap(),
            },
        })
    }
    /// The entire range that is non-editable, up to the nick.
    pub fn non_editable_range(
        &self,
        editor: &Editor,
        pam: Pam<C>,
    ) -> Option<WithOrientation<ContigRange<C>>> {
        let start = pam.v.at.start;
        Some(WithOrientation {
            orientation: pam.orientation,
            v: ContigRange {
                contig: pam.v.contig,
                at: 0..start.checked_sub(editor.nick_distance)?,
            },
        })
    }
    /// The PBS (primer binding site) target is the flap on the same side as the PAM,
    /// it's what will function as a jumping off point for the reverse transcription.
    pub fn primer_binding_site(
        &self,
        editor: &Editor,
        pam: Pam<C>,
        pbs_size: u64,
    ) -> Option<WithOrientation<ContigRange<C>>> {
        let nick_distance = editor.nick_distance.u64_unwrap();

        let end = pam.v.at.start.checked_sub(nick_distance)?;
        let range = end.checked_sub(pbs_size)?..end;

        Some(WithOrientation {
            orientation: pam.orientation,
            v: ContigRange {
                contig: pam.v.contig,
                at: range,
            },
        })
    }
    /// The primer matches the sequence on the opposite side as the PAM.
    pub fn primer(
        &self,
        editor: &Editor,
        pam: Pam<C>,
        pbs_size: u64,
    ) -> Option<WithOrientation<ContigRange<C>>> {
        Some(
            self.primer_binding_site(editor, pam, pbs_size)?
                .flip_orientation(),
        )
    }

    /// The start of the translation frame.
    /// On the sense/RNA-like strand (as opposed to the antisense/template strand).
    pub fn translation_frame_start(&self) -> Option<WithOrientation<ContigPosition<C>>> {
        self.translation_frame_start.clone()
    }
    /// The start of the translation frame in the edited contig.
    /// On the sense/RNA-like strand (as opposed to the antisense/template strand).
    pub fn translation_frame_start_edited(
        &self,
    ) -> Option<WithOrientation<ContigPosition<EditedContig<C>>>> {
        let translation_frame_start = self.translation_frame_start.clone()?;

        let result = self.edited_contig().liftover(translation_frame_start);
        if result.is_none() {
            log::warn!("Unable to lift translation frame into edited contig.\n{self:?}");
        }
        result
    }

    /// Potential silent mutations in the original sequence.
    ///
    /// NOTE: in general you will want to use [Self::edited_silent_mutations] instead,
    /// as you'll want to account for any other changes you are making,
    /// including alternatives to the main edit and frame shifts.
    pub fn original_silent_mutations(
        &self,
    ) -> Option<impl Iterator<Item = SilentMutation<C, DnaBase>>> {
        Some(
            self.original()
                .silent_mutations(self.translation_frame_start.clone()?)
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    /// Potential silent mutations in the edited sequence.
    pub fn edited_silent_mutations(
        &self,
    ) -> Option<impl Iterator<Item = SilentMutation<EditedContig<C>, DnaBase>>> {
        Some(
            self.edited()
                .silent_mutations(self.translation_frame_start_edited()?)
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}
impl<C> Edit<C>
where
    C: Contig + Clone,
{
    /// The ranges that are editable, but excluding the seed, pam, and the region that is already being edited.
    /// Capped at `cap` away from the edit.
    // TODO: test
    pub(super) fn _mmr_evading_ranges(
        &self,
        pam: Pam<C>,
        cap: u64,
    ) -> Option<impl Iterator<Item = WithOrientation<ContigRange<C>>>> {
        let mut edit_range = self.edit_range_in_original();
        edit_range.set_orientation(pam.orientation);

        let head = {
            let orientation = edit_range.orientation;
            let contig = edit_range.v.contig.clone();
            // From the end of the pam to the beginning of the edit
            let at = pam.v.at.end..Ord::min(pam.v.at.end + cap, edit_range.v.at.start);
            WithOrientation {
                orientation,
                v: ContigRange { contig, at },
            }
        };

        let tail = {
            let WithOrientation {
                orientation,
                v: ContigRange { contig, at },
            } = edit_range.clone();
            // From the end of the edit range onward.
            let at = at.end..Ord::min(at.end + cap, self.original_len().u64_unwrap());
            WithOrientation {
                orientation,
                v: ContigRange { contig, at },
            }
        };

        Some([head, tail].into_iter().filter(|r| !r.v.is_empty()))
    }
}

pub(super) fn cached_regex(regex: &IupacDnaSequenceSlice) -> Arc<Regex> {
    static PAM_REGEX: LazyLock<Arc<Regex>> = LazyLock::new(|| {
        let sequence = Editor::sp_cas9().pam_pattern.compile_regex::<DnaBase>();
        Arc::new(Regex::new(&sequence).unwrap())
    });
    static OTHER: Mutex<BTreeMap<String, Arc<Regex>>> = Mutex::new(BTreeMap::new());

    if **regex == [IupacDnaBase::N, IupacDnaBase::G, IupacDnaBase::G] {
        return PAM_REGEX.clone();
    }

    let regex = regex.compile_regex::<DnaBase>();
    let mut other = OTHER.lock().unwrap();
    other
        .entry(regex.clone())
        .or_insert_with(|| Arc::new(Regex::new(&regex).unwrap()))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    use biocore::genome::ArcContig;
    use utile::num::TryU64;

    const EDIT_REGEX: &str = r"[ACGT]{0,50}\([ACGT]{0,5}/[ACGT]{0,5}\)[ACGT]{0,50}";

    proptest::proptest! {
        #[test]
        fn test_parse_edit(edit in EDIT_REGEX) {
            Edit::parse(&edit).unwrap();
        }

        #[test]
        fn test_no_panic(edit in EDIT_REGEX) {
            run_all_methods(&Edit::parse(&edit).unwrap());
        }

        #[test]
        fn test_invariants(edit in EDIT_REGEX) {
            invariants(&Edit::parse(&edit).unwrap());
        }
    }

    fn run_all_methods(edit: &Edit<ArcContig>) {
        let editor = Editor::sp_cas9();
        let _ = edit.edit_range_in_original();
        let _ = edit.edit_range_in_edited();
        let _ = edit.original_len();
        let _ = edit.edited_len();
        let _ = edit.original_contig();
        let _ = edit.edited_contig();
        let _ = edit.original();
        let _ = edit.edited();
        edit.diff_edit_range().for_each(drop);
        let pams = edit.pams(&editor);
        for pam in pams {
            let _ = edit.nick(&editor, pam.clone());
            let _ = edit.spacer(&editor, pam.clone());
            let _ = edit.editable_seed(&editor, pam.clone());
            let _ = edit.primer_binding_site(&editor, pam.clone(), 10);
            let _ = edit.primer(&editor, pam.clone(), 10);
        }
    }

    fn invariants(edit: &Edit<ArcContig>) {
        let editor = Editor::sp_cas9();

        let pams = edit.pams(&editor);
        for pam in pams {
            assert!(pam.v.at.start < edit.original_len().u64_unwrap());
            assert!(pam.v.at.end <= edit.original_len().u64_unwrap());

            let seq = edit.get_original(pam.clone());
            assert_eq!(seq.len(), 3);
            assert_eq!(&seq.to_string()[1..3], "GG");

            if let Some(nick) = edit.nick(&editor, pam.clone()) {
                let expected_nick = pam.v.at.start.checked_sub(editor.nick_distance);
                assert_eq!(Some(nick.v.at), expected_nick);
                assert_eq!(nick.orientation, pam.orientation);

                let non_editable = edit.non_editable_range(&editor, pam.clone()).unwrap();
                let editable = edit.editable_range(&editor, pam.clone()).unwrap();
                assert_eq!(non_editable.v.at.end, editable.v.at.start);
                assert_eq!(non_editable.v.at.start, 0);
                assert_eq!(editable.v.at.end, edit.original_len().u64_unwrap());

                let nick_in_edited = edit.nick_in_edited(&editor, pam.clone()).unwrap();
                assert_eq!(nick.v.at, nick_in_edited.v.at);
                assert_eq!(nick.orientation, nick_in_edited.orientation);
            } else {
                assert!(pam.v.at.start < editor.nick_distance);
            }

            if let Some(spacer) = edit.spacer(&editor, pam.clone()) {
                assert_eq!(spacer.v.len(), editor.spacer_size);
                assert_eq!(spacer.v.at.end, pam.v.at.start);
                assert_eq!(spacer.orientation, pam.orientation);
                let target = edit.cas_target(&editor, pam.clone()).unwrap();
                assert_eq!(spacer, target.flip_orientation());
            } else {
                assert!(pam.v.at.start < editor.spacer_size);
            }

            if let Some(seed) = edit.editable_seed(&editor, pam.clone()) {
                assert_eq!(seed.v.len(), editor.nick_distance);
                assert_eq!(seed.v.at.end, pam.v.at.start);
                assert_eq!(seed.orientation, pam.orientation);
            }

            for pbs_size in [8, 10, 13, 15] {
                if let Some(pbs) = edit.primer_binding_site(&editor, pam.clone(), pbs_size) {
                    assert_eq!(pbs.v.len(), pbs_size);
                    assert_eq!(pbs.orientation, pam.orientation);

                    let nick = edit.nick(&editor, pam.clone()).unwrap();
                    assert_eq!(pbs.v.at.end, nick.v.at);
                } else {
                    assert!(pam.v.at.start < editor.nick_distance + pbs_size);
                }
            }
        }
    }

    #[test]
    fn test_pam_finding_sanity_check() {
        let editor = Editor::sp_cas9();

        let edit = Edit::parse("TTTTTTGGGTTT(/)TTTCCCTTTTTT").unwrap();
        let pams = edit.pams(&editor);

        assert_eq!(
            pams,
            vec![
                WithOrientation {
                    orientation: SequenceOrientation::Forward,
                    v: ContigRange {
                        contig: edit.original_contig(),
                        at: 5..8
                    }
                },
                WithOrientation {
                    orientation: SequenceOrientation::Forward,
                    v: ContigRange {
                        contig: edit.original_contig(),
                        at: 6..9
                    }
                },
                WithOrientation {
                    orientation: SequenceOrientation::Reverse,
                    v: ContigRange {
                        contig: edit.original_contig(),
                        at: 5..8
                    }
                },
                WithOrientation {
                    orientation: SequenceOrientation::Reverse,
                    v: ContigRange {
                        contig: edit.original_contig(),
                        at: 6..9
                    }
                }
            ]
        );

        let short_edit = Edit::parse("AGGA(C/T)").unwrap();
        let pams = short_edit.pams(&editor);
        assert_eq!(pams.len(), 1);

        let no_pam = Edit::parse("AAAAAAAAAAAAAAAAAAAA(C/T)AAAAAAAAAAAAAAAA").unwrap();
        let pams = no_pam.pams(&editor);
        assert_eq!(pams.len(), 0);

        let overlap = Edit::parse("AAAAAAAAAGGGGAAAAAAAAAAA(C/T)AAAAAAAAAAA").unwrap();
        let pams = overlap.pams(&editor);
        assert_eq!(pams.len(), 3);
    }
}
