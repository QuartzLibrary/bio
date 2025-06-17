pub mod cli;
pub mod web;

use std::collections::{hash_map::Entry, HashMap};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use biocore::location::{
    GenomePosition, GenomeRange, LocationConversionError, SequenceOrientation,
};

/// Manually checked lowest exponent before it starts to fail.
const MIN_MATCH_FALLBACK: f64 = 1e-45;
const CONVERSION_ERROR: &str = "
Failed to recover the output position from range.
This usually happens because the liftover utility has merged into 
a single range multiple positions the original position maps to.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(thiserror::Error)]
pub enum FailureReason {
    // /// Deleted in new:
    // ///     Sequence intersects no chains
    // Deleted, // Just return an empty vector instead.
    /// Partially deleted in new:
    ///     Sequence insufficiently intersects one chain
    #[error("Partially deleted in new: Sequence insufficiently intersects one chain")]
    PartiallyDeleted,
    /// Split in new:
    ///     Sequence insufficiently intersects multiple chains
    #[error("Split in new: Sequence insufficiently intersects multiple chains")]
    Split,
    /// Duplicated in new:
    ///     Sequence sufficiently intersects multiple chains
    #[error("Duplicated in new: Sequence sufficiently intersects multiple chains")]
    Duplicated,
    /// Boundary problem:
    ///     Missing start or end base in an exon
    #[error("Boundary problem: Missing start or end base in an exon")]
    BoundaryProblem,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(thiserror::Error)]
pub enum PositionFailureReason {
    #[error(transparent)]
    Liftover(#[from] FailureReason),
    /// The liftover utility may map the position to multiple locations
    /// and then merge them into a single range, preventing the conversion
    /// back to individual positions.
    #[error("{CONVERSION_ERROR}\n:Original position: {from:?}\nResulting range: {to:?}")]
    Conversion {
        from: GenomePosition,
        to: GenomeRange,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UcscLiftoverSettings {
    /// Minimum ratio of bases that must remap
    ///
    /// NOTE: if you set this to 0, it will be converted to [MIN_MATCH_FALLBACK]
    /// because the server seems to sometimes fail with something like
    /// "Chain mapping error: chr1:129374201-129374202" otherwise.
    pub min_match: NotNan<f64>,

    // // Keep original positions in output
    // pub keep_original_positions: bool, // Always true

    // Regions defined by chrom:start-end (BED 4 to BED 6)
    // /// Allow multiple output regions
    // pub multi_region_allowed: bool, // Always true
    /// Minimum hit size in query
    pub min_query: u64,
    /// Minimum chain size in target
    pub min_chain: u64,

    // Regions with an exon-intron structure (usually transcripts, BED 12)
    /// Minimum ratio of alignment blocks or exons that must map
    pub min_blocks: u64,
    // /// If exon is not mapped, use the closest mapped base
    // pub is_thick_fudge_set: bool,
}
impl Default for UcscLiftoverSettings {
    fn default() -> Self {
        Self {
            min_match: NotNan::new(0.95).unwrap(),
            // keep_original_positions: false,
            // multi_region_allowed: true,
            min_query: 0,
            min_chain: 0,
            min_blocks: 1,
            // is_thick_fudge_set: false,
        }
    }
}
impl UcscLiftoverSettings {
    pub fn loose() -> Self {
        Self {
            min_match: NotNan::new(0.).unwrap(), // See docs, will fallback to 1e-45
            ..Default::default()
        }
    }
    fn preprocess(mut self) -> Self {
        if self.min_match == 0. {
            self.min_match = NotNan::new(MIN_MATCH_FALLBACK).unwrap();
        }
        self
    }
}

fn parse_success_file(res: &str) -> std::io::Result<Vec<(GenomeRange, GenomeRange, u64)>> {
    res.split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let error = || {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid format: {s}"),
                )
            };

            let [chr, start, end, old, i] = &*s.split('\t').collect::<Vec<_>>() else {
                return Err(error());
            };
            let from_range = {
                let (name, rest) = old.split_once(':').ok_or(error())?;
                let (from, to) = rest.split_once('-').ok_or(error())?;
                let from: u64 = from.parse().map_err(utile::io::invalid_data)?;
                let to: u64 = to.parse().map_err(utile::io::invalid_data)?;
                GenomeRange {
                    name: name.to_owned(),
                    at: (from - 1)..to,
                    orientation: SequenceOrientation::Forward,
                }
            };
            let to_range = {
                let start: u64 = start.parse().map_err(utile::io::invalid_data)?;
                let end: u64 = end.parse().map_err(utile::io::invalid_data)?;
                GenomeRange {
                    name: (*chr).to_owned(),
                    at: start..end,
                    orientation: SequenceOrientation::Forward,
                }
            };
            let i: u64 = i.parse().map_err(utile::io::invalid_data)?;
            Ok((from_range, to_range, i))
        })
        .try_collect()
}
fn parse_failure_file(res: &str) -> std::io::Result<Vec<(GenomeRange, Option<FailureReason>)>> {
    // Deleted in new:
    //     Sequence intersects no chains
    // Partially deleted in new:
    //     Sequence insufficiently intersects one chain
    // Split in new:
    //     Sequence insufficiently intersects multiple chains
    // Duplicated in new:
    //     Sequence sufficiently intersects multiple chains
    // Boundary problem:
    //     Missing start or end base in an exon

    res.split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|lines| {
            let [line1, line2] = lines else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Expected two lines, got {lines:?} instead."),
                ));
            };

            let error = || {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid format: {line1}\n{line2}"),
                )
            };

            let reason = match *line1 {
                "#Deleted in new" => None,
                "#Partially deleted in new" => Some(FailureReason::PartiallyDeleted),
                "#Split in new" => Some(FailureReason::Split),
                "#Duplicated in new" => Some(FailureReason::Duplicated),
                "#Boundary problem" => Some(FailureReason::BoundaryProblem),
                _ => return Err(error()),
            };

            let [chr, start, end] = &*line2.split('\t').collect::<Vec<_>>() else {
                return Err(error());
            };

            let location = {
                let start: u64 = start.parse().map_err(utile::io::invalid_data)?;
                let end: u64 = end.parse().map_err(utile::io::invalid_data)?;

                GenomeRange {
                    name: (*chr).to_owned(),
                    at: start..end,
                    orientation: SequenceOrientation::Forward,
                }
            };

            Ok((location, reason))
        })
        .try_collect()
}
pub(super) fn combine_success_and_failure(
    locations: &[GenomeRange],
    success: Option<Vec<(GenomeRange, GenomeRange, u64)>>,
    failure: Option<Vec<(GenomeRange, Option<FailureReason>)>>,
) -> std::io::Result<Vec<Result<Vec<GenomeRange>, FailureReason>>> {
    let success = success.map(|success| {
        let mut result: HashMap<GenomeRange, Vec<GenomeRange>> = HashMap::new();

        for (from, to, _i) in success {
            // `i` is the index/order of the mapped region.
            // As multiple mapped ranges are reported from the same input range, the number increases.
            // Since we use the explicitly spelled out source range to figure out what maps to what,
            // we don't need this number. (Also, we would need to interleave the errors anyway.)

            result.entry(from).or_default().push(to);
        }

        for vec in result.values_mut() {
            vec.sort();
            vec.dedup(); // A range might appear in the input multiple times.
        }

        result
    });

    let failure = failure.map(|failure| {
        let mut result: HashMap<GenomeRange, Option<FailureReason>> = HashMap::new();

        for (loc, reason) in failure {
            match result.entry(loc) {
                Entry::Occupied(occupied_entry) => assert_eq!(*occupied_entry.get(), reason),
                Entry::Vacant(vacant_entry) => drop(vacant_entry.insert(reason)),
            }
        }

        result
    });

    #[cfg(debug_assertions)]
    {
        use std::collections::HashSet;

        assert!(success.is_some() || failure.is_some());
        let query_unique_count = locations.iter().collect::<HashSet<_>>().len();
        let success_unique_count = success.as_ref().map(|success| success.len()).unwrap_or(0);
        let failure_unique_count = failure.as_ref().map(|failure| failure.len()).unwrap_or(0);
        assert_eq!(
            query_unique_count,
            success_unique_count + failure_unique_count
        );
    }

    match (success, failure) {
        (None, None) => unreachable!(),
        (None, Some(failure)) => Ok(locations
            .iter()
            .map(|loc| match *failure.get(loc).unwrap() {
                Some(failure_reason) => Err(failure_reason),
                None => Ok(vec![]),
            })
            .collect()),
        (Some(success), None) => Ok(locations
            .iter()
            .map(|loc| Ok(success.get(loc).unwrap().clone()))
            .collect()),
        (Some(success), Some(failure)) => locations
            .iter()
            .map(
                |loc| match (success.get(loc).cloned(), failure.get(loc).copied()) {
                    (None, None) => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("The location was not returned {loc:?}."),
                    )),
                    (Some(to), None) => Ok(Ok(to)),
                    (None, Some(None)) => Ok(Ok(vec![])),
                    (None, Some(Some(reason))) => Ok(Err(reason)),
                    (Some(to), Some(reason)) => {
                        unreachable!(
                            "The location was returned twice {loc:?} ({to:?} and {reason:?})."
                        )
                    }
                },
            )
            .try_collect(),
    }
}

fn recover_positions(
    original: &GenomePosition,
    v: Vec<GenomeRange>,
) -> Result<Vec<GenomePosition>, PositionFailureReason> {
    v.into_iter()
        .map(|v| {
            v.try_into().map_err(|LocationConversionError { from }| {
                PositionFailureReason::Conversion {
                    from: original.clone(),
                    to: from,
                }
            })
        })
        .try_collect()
}
