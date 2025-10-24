use std::{
    collections::BTreeMap,
    ops::Range,
    sync::{Arc, LazyLock, Mutex},
};

use biocore::{
    dna::{DnaBase, DnaSequence, IupacDnaBase, IupacDnaSequenceSlice},
    genome::{ArcContig, ContigRef, EditedContig},
    location::orientation::{SequenceOrientation, WithOrientation},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use utile::{
    num::{TryI64, TryU64},
    range::RangeLen,
    regex_ext::find_iter_overlapping,
};

use crate::{ContigPosition, ContigRange, SilentMutation, editor::Editor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Edit {
    start: DnaSequence,
    original: DnaSequence,
    edited: DnaSequence,
    end: DnaSequence,

    /// On the sense/RNA-like strand (as opposed to the antisense/template strand).
    pub translation_frame_start: Option<WithOrientation<ContigPosition>>,
}
impl Edit {
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

        Some(Self {
            start: start.parse().ok()?,
            original: original.parse().ok()?,
            edited: edited.parse().ok()?,
            end: end.parse().ok()?,

            translation_frame_start: None,
        })
    }

    pub fn original_len(&self) -> usize {
        self.start.len() + self.original.len() + self.end.len()
    }
    pub fn original_contig(&self) -> ArcContig {
        ArcContig::from_contig(ContigRef::new("original", self.original_len() as u64))
    }
    pub fn original(&self) -> DnaSequence {
        let Self {
            start,
            original,
            edited: _,
            end,
            translation_frame_start: _,
        } = self;
        format!("{start}{original}{end}").parse().unwrap()
    }
    pub fn edit_range_in_original(&self) -> WithOrientation<ContigRange> {
        let Self {
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
    pub fn get_original(&self, range: WithOrientation<ContigRange>) -> DnaSequence {
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
    pub fn edited_contig(&self) -> EditedContig<ArcContig> {
        let original_contig = self.original_contig();
        let remove = self.edit_range_in_original().v;
        let insert = self.raw_edit_range_in_edited().range_len();
        EditedContig::new(original_contig, remove, insert)
    }
    pub fn edited(&self) -> DnaSequence {
        let Self {
            start,
            original: _,
            edited,
            end,
            translation_frame_start: _,
        } = self;
        format!("{start}{edited}{end}").parse().unwrap()
    }
    pub fn edit_range_in_edited(&self) -> WithOrientation<ContigRange> {
        let range = self.raw_edit_range_in_edited();
        WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigRange {
                contig: ArcContig::from_contig(self.edited_contig()),
                at: range,
            },
        }
    }
    fn raw_edit_range_in_edited(&self) -> Range<u64> {
        let Self {
            start,
            original: _,
            edited,
            end: _,
            translation_frame_start: _,
        } = self;
        let range = start.len()..start.len() + edited.len();
        range.start.u64_unwrap()..range.end.u64_unwrap()
    }
    pub fn get_edited(&self, range: WithOrientation<ContigRange>) -> DnaSequence {
        assert_eq!(ArcContig::from_contig(self.edited_contig()), range.v.contig);
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

    pub fn closest_pam(&self, editor: &Editor) -> Option<WithOrientation<ContigRange>> {
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
    pub fn distance_from_edit(&self, pam: WithOrientation<ContigRange>) -> i64 {
        let edit_start = self
            .edit_range_in_original()
            .into_orientation(pam.orientation)
            .v
            .at
            .start
            .i64_unwrap();

        edit_start - pam.v.at.start.i64_unwrap()
    }

    /// Returns the PAM locations in the original sequence, all ranges are 5' → 3' (and annotated with the orientation).
    pub fn pams(&self, editor: &Editor) -> Vec<WithOrientation<ContigRange>> {
        let Editor {
            pam_pattern,
            nick_distance,
            spacer_size: _,
            scaffold: _,
        } = editor;
        let pam_size = editor.pam_size();
        let nick_distance = *nick_distance;

        let original = self.original();

        if original.len().u64_unwrap() < nick_distance + pam_size {
            return vec![];
        }

        let edit_range = self.edit_range_in_original();

        let mut forward = edit_range.clone().into_start().v;
        let mut reverse = edit_range.flip_orientation().into_start().v;

        // We can go slightly past the edit range because the nick is before the PAM.
        forward += nick_distance;
        reverse += nick_distance;
        // And we include the PAM size so that the regex can find it even if at the end.
        forward += pam_size;
        reverse += pam_size;

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
    /// The nick is on the same strand as the PAM, the side the spacer anneals to.
    pub fn nick(
        &self,
        editor: &Editor,
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigPosition>> {
        Some(WithOrientation {
            orientation: pam.orientation,
            v: pam.into_start().v.checked_sub(editor.nick_distance)?,
        })
    }
    pub fn nick_in_edited(
        &self,
        editor: &Editor,
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigPosition>> {
        let contig = ArcContig::from_contig(self.edited_contig());
        let mut nick = self.nick(editor, pam)?;
        // Since the nick is where the edit start, the position doesn't change and we can just update the contig.
        nick.v.contig = contig;
        Some(nick)
    }
    /// The spacer matches the sequence on the same side as the PAM, and will anneal to the opposite side.
    pub fn spacer(
        &self,
        editor: &Editor,
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigRange>> {
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
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigRange>> {
        Some(self.spacer(editor, pam)?.flip_orientation())
    }
    /// The section between the nick and the PAM.
    pub fn editable_seed(
        &self,
        editor: &Editor,
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigRange>> {
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
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigRange>> {
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
        pam: WithOrientation<ContigRange>,
    ) -> Option<WithOrientation<ContigRange>> {
        let start = pam.v.at.start;
        Some(WithOrientation {
            orientation: pam.orientation,
            v: ContigRange {
                contig: pam.v.contig,
                at: 0..start.checked_sub(editor.nick_distance)?,
            },
        })
    }
    /// The ranges that are editable, but excluding the seed, pam, and the region that is already being edited.
    /// Capped at `cap` away from the edit.
    pub fn mmr_evading_ranges(
        &self,
        pam: WithOrientation<ContigRange>,
        cap: u64,
    ) -> Option<impl Iterator<Item = WithOrientation<ContigRange>>> {
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
    /// The PBS (primer binding site) target is the flap on the same side as the PAM,
    /// it's what will function as a jumping off point for the reverse transcription.
    pub fn primer_binding_site(
        &self,
        editor: &Editor,
        pam: WithOrientation<ContigRange>,
        pbs_size: u64,
    ) -> Option<WithOrientation<ContigRange>> {
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
        pam: WithOrientation<ContigRange>,
        pbs_size: u64,
    ) -> Option<WithOrientation<ContigRange>> {
        Some(
            self.primer_binding_site(editor, pam, pbs_size)?
                .flip_orientation(),
        )
    }

    pub fn translation_frame_start_edited(&self) -> Option<WithOrientation<ContigPosition>> {
        let translation_frame_start = self.translation_frame_start.clone()?;

        let result = self.edited_contig().liftover(translation_frame_start);
        if result.is_none() {
            log::warn!("Unable to lift translation frame into edited contig.\n{self:?}");
        }

        Some(result?.map_value(|p| p.map_contig(ArcContig::from_contig)))
    }
    pub fn original_silent_mutations(&self) -> Option<impl Iterator<Item = SilentMutation>> {
        Some(
            self.original()
                .silent_mutations(self.translation_frame_start.clone()?)
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    pub fn edited_silent_mutations(&self) -> Option<impl Iterator<Item = SilentMutation>> {
        Some(
            self.edited()
                .silent_mutations(self.translation_frame_start_edited()?)
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

fn cached_regex(regex: &IupacDnaSequenceSlice) -> Arc<Regex> {
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
