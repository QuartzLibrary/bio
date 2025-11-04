use std::{fmt, iter, ops::Range};

use either::Either;
use serde::{Deserialize, Serialize};
use utile::{num::TryU64, range::RangeExt};

use crate::{
    dna::{Complement, DnaBase},
    genome::Contig,
    location::{
        ContigPosition, ContigRange,
        orientation::{SequenceOrientation, Stranded},
    },
    sequence::{Sequence, SequenceSlice},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct SilentMutation<C, B> {
    pub at: Stranded<ContigPosition<C>>,
    pub original: [B; 3],
    pub edited: [B; 3],
}
impl<C, B> SilentMutation<C, B>
where
    C: PartialEq + fmt::Debug, // For assertions
    B: PartialEq + fmt::Debug,
{
    pub fn apply(&self, sequence: &mut SequenceSlice<B>)
    where
        C: Contig + Clone,
        B: Complement + Clone,
    {
        match self.at.orientation {
            SequenceOrientation::Forward => {
                let slice = &mut sequence[self.at_range().v];
                assert_eq!(slice.len(), 3);
                assert_eq!(&**slice, &self.original);
                slice.clone_from_slice(&self.edited);
            }
            SequenceOrientation::Reverse => {
                let mut range = self.at_range();
                range.set_orientation(SequenceOrientation::Forward);
                let slice = &mut sequence[range.v];
                assert_eq!(slice.len(), 3);
                assert_eq!(
                    slice,
                    &*Sequence::new(self.original.to_vec()).reverse_complement()
                );
                slice.clone_from_slice(&Sequence::new(self.edited.to_vec()).reverse_complement());
            }
        }
    }
    pub fn affects_range(&self, range: Stranded<ContigRange<C>>) -> bool
    where
        C: Contig + Clone,
        B: Clone,
    {
        assert_eq!(self.at.v.contig, range.v.contig);
        self.affected_positions().any(|pos| range.contains(&pos))
    }
    pub fn affected_positions(&self) -> impl Iterator<Item = Stranded<ContigPosition<C>>>
    where
        C: Contig + Clone,
        B: Clone,
    {
        self.at_range()
            .iter_positions()
            .zip(self.original.clone())
            .zip(self.edited.clone())
            .filter(|((_, o), e)| o != e)
            .map(|((at, _), _)| at)
    }
    pub fn affected_range(&self) -> Stranded<ContigRange<C>>
    where
        C: Clone,
    {
        let range = self.raw_affected_range();
        self.at
            .clone()
            .map_value(|ContigPosition { contig, at }| ContigRange {
                contig,
                at: (at + range.start)..(at + range.end),
            })
    }
    fn raw_affected_range(&self) -> Range<u64>
    where
        C: Clone,
    {
        let mut min = 3;
        let mut max = 0;
        let diff = self
            .original
            .iter()
            .zip(self.edited.iter())
            .enumerate()
            .filter(|(_, (o, e))| o != e)
            .map(|(i, _)| i);
        for i in diff {
            min = min.min(i);
            max = max.max(i + 1);
        }
        let range = min.u64_unwrap()..max.u64_unwrap();
        assert!((0..3).contains_range(&range));
        if range.is_empty() { 0..0 } else { range }
    }
    pub fn at_range(&self) -> Stranded<ContigRange<C>>
    where
        C: Clone,
    {
        let Stranded {
            orientation,
            v: ContigPosition { contig, at },
        } = self.at.clone();
        Stranded {
            orientation,
            v: ContigRange {
                contig,
                at: at..at + 3,
            },
        }
    }
}

// TODO: make it for all base types.
// TODO: subset-aware sequence type (it knows which contig subset it represents)
impl SequenceSlice<DnaBase> {
    /// Returns an iterator over all the [SilentMutation]s in the sequence.
    ///
    /// Uses the standard codon map ([crate::aminoacid::codons::map::STANDARD]).
    ///
    /// NOTE: mutations are returned in order they appear on the forward strand.
    /// NOTE: Assumes the sequence is of the forward strand.
    pub fn silent_mutations<C: Contig + Clone>(
        &self,
        frame_start: Stranded<ContigPosition<C>>,
    ) -> impl Iterator<Item = SilentMutation<C, DnaBase>> + use<'_, C> {
        assert_eq!(frame_start.v.contig.size(), self.len().u64_unwrap()); // TODO: relax constraint.

        let pack = {
            let contig = frame_start.clone();
            move |(i, original, edited): (u64, [DnaBase; 3], [DnaBase; 3])| SilentMutation {
                at: contig.clone().map_value(|v| v + i),
                original,
                edited,
            }
        };

        if frame_start.orientation.is_forward() {
            Either::Left(self[frame_start.v..].raw_silent_mutations().map(pack))
        } else {
            // TODO avoid re-allocating the reverse complement, especially on the whole sequence.
            Either::Right(
                self.reverse_complement()[frame_start.v..]
                    .raw_silent_mutations()
                    .map(pack)
                    .map(|mut m| {
                        m.at = m.at.into_reverse();
                        m
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev(),
            )
        }
    }
    /// Returns an iterator over all the silent mutation positions, the original codon, and the synonym codon.
    fn raw_silent_mutations(
        &self,
    ) -> impl Iterator<Item = (u64, [DnaBase; 3], [DnaBase; 3])> + use<'_> {
        self.chunks(3)
            .enumerate()
            .filter_map(|(i, codon)| {
                // Skip the last one if shorter than 3 bases.
                let codon: [DnaBase; 3] = codon.try_into().ok()?;

                let synonyms = crate::aminoacid::codons::map::STANDARD_SYNONYMS
                    .get(&codon)
                    .clone();

                Some(
                    iter::repeat((i * 3).u64_unwrap())
                        .zip(iter::repeat(codon))
                        .zip(synonyms)
                        .map(|((i, codon), synonym)| (i, codon, synonym)),
                )
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use utile::vec::VecExt;

    use super::*;
    use crate::{
        aminoacid::{
            ProteinSequence,
            codons::map::{STANDARD, STANDARD_SYNONYMS},
        },
        dna::{DnaBase, DnaSequence},
        genome::ContigRef,
        location::{
            ContigRange,
            orientation::{SequenceOrientation, Stranded},
        },
    };

    fn seq_from_codons(codons: &[[DnaBase; 3]]) -> DnaSequence {
        codons.iter().copied().flatten().collect()
    }
    // A sequence containing all 64 codons in some order.
    fn all_codons() -> DnaSequence {
        let all_codons: Vec<[DnaBase; 3]> = STANDARD.iter().map(|(c, _)| *c).collect();
        seq_from_codons(&all_codons)
    }

    #[test]
    fn sanity_checks() {
        let seq = all_codons();
        let raw: Vec<(u64, [DnaBase; 3], [DnaBase; 3])> = seq.raw_silent_mutations().collect();

        for (i, orig, edit) in &raw {
            // Starts at codon boundaries and within bounds.
            assert_eq!(i % 3, 0);
            assert!(*i + 2 < seq.len().u64_unwrap());

            // Edited is a true synonym.
            let aa_orig = STANDARD.get(orig);
            let aa_edit = STANDARD.get(edit);
            assert_eq!(aa_orig, aa_edit);
            assert!(STANDARD_SYNONYMS.get(orig).contains(edit));

            // Must actually change at least one base.
            assert_ne!(orig, edit);
        }

        let seq2: DnaSequence = "TA".parse().unwrap();
        assert_eq!(seq2.raw_silent_mutations().count(), 0);
    }

    #[test]
    fn oriented_enumeration_and_apply_equivalence() {
        const CONTIG: &str = "test_contig";

        // Use a compact sequence with multiple codons including those with synonyms.
        //                 Ala-Leu-Arg
        const SEQ: &str = "GCT-CTT-CGT";
        const SEQ_EXT: &str = "A-GCT-CTT-CGT-A";
        const SEQ_AA: &str = "ALR";

        //                    Thr-Lys-Ser
        const SEQ_RC: &str = "ACG-AAG-AGC";
        const SEQ_AA_RC: &str = "TKS";

        let contig = ContigRef::new(CONTIG, SEQ.replace('-', "").len().u64_unwrap());
        let contig_ext = ContigRef::new(CONTIG, SEQ_EXT.replace('-', "").len().u64_unwrap());

        let to_synonyms = |i: usize, codon: [DnaBase; 3], orientation: SequenceOrientation| {
            let at = Stranded {
                orientation,
                v: contig.at(i.u64_unwrap()),
            };
            STANDARD_SYNONYMS
                .get(&codon)
                .clone()
                .into_iter()
                .map(move |synonym| SilentMutation {
                    at,
                    original: codon,
                    edited: synonym,
                })
        };

        let seq: DnaSequence = SEQ.replace('-', "").parse().unwrap();
        let seq_ext: DnaSequence = SEQ_EXT.replace('-', "").parse().unwrap();

        // Check sequences
        {
            let seq_aa: ProteinSequence = SEQ_AA.parse().unwrap();
            assert_eq!(seq_aa, seq.to_amino_acids());

            let seq_rc: DnaSequence = SEQ_RC.replace('-', "").parse().unwrap();
            let seq_aa_rc: ProteinSequence = SEQ_AA_RC.parse().unwrap();
            assert_eq!(seq_aa_rc, seq_rc.to_amino_acids());

            assert_eq!(seq.clone().reverse_complement(), seq_rc);
        }

        {
            let start = Stranded::new_forward(contig.at(0));

            assert_eq!(
                seq.silent_mutations(start).collect::<Vec<_>>(),
                seq_ext
                    .silent_mutations(Stranded::new_forward(contig_ext.at(1)))
                    .map(|mut m| {
                        m.at.v.contig = contig;
                        m.at.v.at -= 1;
                        m
                    })
                    .collect::<Vec<_>>(),
            );

            assert_eq!(
                seq.silent_mutations(start).collect::<Vec<_>>(),
                [0, 3, 6]
                    .into_iter()
                    .map(|i| {
                        let codon: &[DnaBase; 3] = (&*seq[i..i + 3]).try_into().unwrap();
                        (i, codon)
                    })
                    .flat_map(|(i, codon)| to_synonyms(i, *codon, SequenceOrientation::Forward))
                    .collect::<Vec<_>>()
            );

            assert_eq!(
                seq.silent_mutations(start.map_value(|v| v + 3))
                    .collect::<Vec<_>>(),
                [3, 6]
                    .into_iter()
                    .map(|i| {
                        let codon: &[DnaBase; 3] = (&*seq[i..i + 3]).try_into().unwrap();
                        (i, codon)
                    })
                    .flat_map(|(i, codon)| to_synonyms(i, *codon, SequenceOrientation::Forward))
                    .collect::<Vec<_>>()
            );
        }

        {
            let seq_rc: DnaSequence = SEQ_RC.replace('-', "").parse().unwrap();

            let start_rc = Stranded::new_reverse(contig.at(0));

            assert_eq!(
                seq.silent_mutations(start_rc).collect::<Vec<_>>(),
                seq_ext
                    .silent_mutations(Stranded::new_reverse(contig_ext.at(1)))
                    .map(|mut m| {
                        m.at.v.contig = contig;
                        m.at.v.at -= 1;
                        m
                    })
                    .collect::<Vec<_>>(),
            );

            assert_eq!(
                seq.silent_mutations(start_rc).collect::<Vec<_>>(),
                [0, 3, 6]
                    .into_iter()
                    .map(|i| {
                        let codon: &[DnaBase; 3] = (&*seq_rc[i..i + 3]).try_into().unwrap();
                        (i, codon)
                    })
                    .flat_map(|(i, codon)| to_synonyms(i, *codon, SequenceOrientation::Reverse))
                    .collect::<Vec<_>>()
                    .reversed()
            );

            assert_eq!(
                seq.silent_mutations(start_rc.map_value(|v| v + 3))
                    .collect::<Vec<_>>(),
                [3, 6]
                    .into_iter()
                    .map(|i| {
                        let codon: &[DnaBase; 3] = (&*seq_rc[i..i + 3]).try_into().unwrap();
                        (i, codon)
                    })
                    .flat_map(|(i, codon)| to_synonyms(i, *codon, SequenceOrientation::Reverse))
                    .collect::<Vec<_>>()
                    .reversed()
            );
        }
    }

    #[test]
    #[expect(clippy::single_range_in_vec_init)]
    fn apply_affects_and_ranges_invariants() {
        let contig = ContigRef::new("test_contig", 3);
        let at = Stranded::new_forward(contig.at(0));

        let new_range = |range: Range<u64>| {
            at.map_value(|ContigPosition { contig, at }| ContigRange {
                contig,
                at: (at + range.start)..(at + range.end),
            })
        };

        let muts: [(_, _, _, &[_], &[_]); _] = [
            ("AAA", "AAA", 0..0, &[], &[0..0, 0..1, 0..3, 2..3, 3..3]),
            ("AAA", "CAA", 0..1, &[0..100, 0..1], &[1..2, 0..0]),
            ("AAA", "ACA", 1..2, &[0..3], &[0..1]),
            ("AAA", "AAC", 2..3, &[], &[]),
            ("AAA", "CCA", 0..2, &[], &[]),
            ("AAA", "ACC", 1..3, &[], &[]),
        ];
        let muts = muts.map(
            |(original, edited, exact_affected_range, affected_ranges, unaffected_ranges)| {
                let original = original.parse::<DnaSequence>().unwrap();
                let original: [DnaBase; 3] = (&**original).try_into().unwrap();
                let edited = edited.parse::<DnaSequence>().unwrap();
                let edited: [DnaBase; 3] = (&**edited).try_into().unwrap();
                (
                    SilentMutation {
                        at,
                        original,
                        edited,
                    },
                    new_range(exact_affected_range),
                    affected_ranges
                        .iter()
                        .map(|r| new_range(r.clone()))
                        .collect::<Vec<_>>(),
                    unaffected_ranges
                        .iter()
                        .map(|r| new_range(r.clone()))
                        .collect::<Vec<_>>(),
                )
            },
        );

        for (m, exact_affected_range, affected_ranges, unaffected_ranges) in muts {
            println!("{m:?}");

            assert_eq!(m.affected_range(), exact_affected_range);
            for affected_range in affected_ranges {
                assert!(
                    m.affects_range(affected_range.clone()),
                    "{affected_range:?}"
                );
            }
            for unaffected_range in unaffected_ranges {
                assert!(
                    !m.affects_range(unaffected_range.clone()),
                    "{unaffected_range:?}"
                );
            }
        }
    }
}
