#![feature(strict_overflow_ops)]
#![feature(iterator_try_collect)]
#![feature(impl_trait_in_assoc_type)]

mod parse;

pub mod bindings;
pub mod sources;

use std::{cmp, collections::BTreeMap, ops::Range};

use utile::range::{RangeExt, RangeLen};

use biocore::location::{GenomePosition, GenomeRange, SequenceOrientation};

/// https://genome.ucsc.edu/goldenPath/help/chain.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Liftover {
    pub chains: Vec<Chain>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    pub header: ChainHeader,
    blocks: Vec<AlignmentBlock>,
    last_block: u64,
}
/// The initial header line starts with the keyword `chain`,
/// followed by 11 required attribute values, and ends with a blank line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainHeader {
    /// Chain score
    pub score: u64,
    /// (reference/target sequence)
    pub t: ChainRange,
    /// (query sequence)
    pub q: ChainRange,
    /// chain ID
    pub id: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainRange {
    /// chromosome size
    pub size: u64,
    pub range: GenomeRange,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlignmentBlock {
    /// the size of the ungapped alignment
    pub size: u64,
    /// the difference between the end of this block and the beginning of the next block (reference/target sequence)
    pub dt: i64,
    /// the difference between the end of this block and the beginning of the next block (query sequence)
    pub dq: u64,
}

impl Liftover {
    pub fn indexed(&self) -> LiftoverIndexed {
        LiftoverIndexed::from_liftover(self)
    }

    pub fn map(&self, loc: GenomePosition) -> impl Iterator<Item = GenomePosition> + use<'_> {
        self.map_raw(loc).map(|(r, _)| r)
    }
    pub fn map_range(&self, range: GenomeRange) -> impl Iterator<Item = GenomeRange> + use<'_> {
        self.map_range_raw(range).map(|(r, _)| r)
    }

    pub fn map_raw(
        &self,
        loc: GenomePosition,
    ) -> impl Iterator<Item = (GenomePosition, bool)> + use<'_> {
        self.chains.iter().flat_map(move |c| c.map_raw(loc.clone()))
    }
    pub fn map_range_raw(
        &self,
        range: GenomeRange,
    ) -> impl Iterator<Item = (GenomeRange, bool)> + use<'_> {
        self.chains
            .iter()
            .flat_map(move |c| c.map_range_raw(range.clone()))
    }

    fn iter_ranges(&self) -> impl Iterator<Item = (ChainRange, ChainRange)> + use<'_> {
        self.chains.iter().flat_map(|c| c.iter_ranges())
    }
}
impl Chain {
    pub fn map_raw(
        &self,
        mut loc: GenomePosition,
    ) -> impl Iterator<Item = (GenomePosition, bool)> + use<'_> {
        let original_orientation = loc.orientation;
        let flip_to_map = loc.orientation != self.header.t.range.orientation;
        loc.set_orientation(self.header.t.range.orientation, self.header.t.size);
        if !self.header.t.contains(&loc) {
            return None.into_iter().flatten();
        }
        let at = loc.at;
        drop(loc);

        let mut t_start = self.header.t.range.at.start;
        let mut q_start = self.header.q.range.at.start;

        let mapped = self
            .blocks()
            .filter_map(move |b| {
                let mut r = None;

                if (t_start..(t_start + b.size)).contains(&at) {
                    let new = GenomePosition {
                        name: self.header.q.range.name.clone(),
                        at: q_start + (at - t_start),
                        orientation: self.header.q.range.orientation,
                    };

                    r = Some(new);
                }

                t_start += b.size;
                q_start += b.size;

                t_start = t_start.strict_add_signed(b.dt);
                q_start += b.dq;

                r
            })
            .map(move |mut r| {
                let flipped = if flip_to_map {
                    // If we flipped before, and do not need to flip again,
                    // we are already on the original strand, but the original reference sequence
                    // for this region is on the opposite strand, so should be fixed.
                    r.orientation == original_orientation
                } else {
                    // If we ended up on the opposite strand, let's make a note that we are flipping back.
                    r.orientation != original_orientation
                };
                r.set_orientation(original_orientation, self.header.q.size);
                (r, flipped)
            });

        Some(mapped).into_iter().flatten()
    }
    pub fn map_range_raw(
        &self,
        range: GenomeRange,
    ) -> impl Iterator<Item = (GenomeRange, bool)> + use<'_> {
        let intersected = match self.header.t.intersect(range.clone()) {
            Some(intersected) if intersected.is_empty() => return None.into_iter().flatten(),
            Some(intersected) => intersected,
            None => return None.into_iter().flatten(),
        };
        assert_eq!(intersected.name, range.name);
        let original_orientation = range.orientation;
        let flip_to_map = range.orientation != self.header.t.range.orientation;
        drop(range);

        let mut t_start = self.header.t.range.at.start;
        let mut q_start = self.header.q.range.at.start;

        let mapped = self
            .blocks()
            .filter_map(move |b| {
                let mut r = None;

                let intersected =
                    (t_start..(t_start + b.size)).intersection(intersected.at.clone());

                if !intersected.is_empty() {
                    let shift = intersected.start - t_start;
                    r = Some(GenomeRange {
                        name: self.header.q.range.name.clone(),
                        at: (q_start + shift)..(q_start + shift + intersected.range_len()),
                        orientation: self.header.q.range.orientation,
                    });
                }

                t_start += b.size;
                q_start += b.size;

                t_start = t_start.strict_add_signed(b.dt);
                q_start += b.dq;

                r
            })
            .map(move |mut r| {
                let flipped = if flip_to_map {
                    // If we flipped before, and do not need to flip again,
                    // we are already on the original strand, but the original reference sequence
                    // for this region is on the opposite strand, so should be fixed.
                    r.orientation == original_orientation
                } else {
                    // If we ended up on the opposite strand, let's make a note that we are flipping back.
                    r.orientation != original_orientation
                };
                r.set_orientation(original_orientation, self.header.q.size);
                (r, flipped)
            });

        Some(mapped).into_iter().flatten()
    }
    pub fn blocks(&self) -> impl Iterator<Item = AlignmentBlock> + use<'_> {
        self.blocks.iter().copied().chain([AlignmentBlock {
            size: self.last_block,
            dt: 0,
            dq: 0,
        }])
    }

    fn iter_ranges(&self) -> impl Iterator<Item = (ChainRange, ChainRange)> + use<'_> {
        let mut t_start = self.header.t.range.at.start;
        let mut q_start = self.header.q.range.at.start;

        self.blocks().map(move |b| {
            let t_fragment = t_start..(t_start + b.size);
            let q_fragment = q_start..(q_start + b.size);

            assert!(!t_fragment.is_empty(), "{t_fragment:?}");
            assert!(!q_fragment.is_empty(), "{q_fragment:?}");

            t_start += b.size;
            q_start += b.size;

            t_start = t_start.strict_add_signed(b.dt);
            q_start += b.dq;

            (
                ChainRange {
                    size: self.header.t.size,
                    range: GenomeRange {
                        name: self.header.t.range.name.clone(),
                        orientation: self.header.t.range.orientation,
                        at: t_fragment,
                    },
                },
                ChainRange {
                    size: self.header.q.size,
                    range: GenomeRange {
                        name: self.header.q.range.name.clone(),
                        orientation: self.header.q.range.orientation,
                        at: q_fragment,
                    },
                },
            )
        })
    }
}
impl ChainRange {
    fn contains(&self, loc: &GenomePosition) -> bool {
        self.range.contains(loc, self.size)
    }
    fn intersect(&self, b: GenomeRange) -> Option<GenomeRange> {
        self.range.intersect(b, self.size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftoverIndexed {
    chromosomes: BTreeMap<String, (Vec<LiftoverIndexedEntry>, u64)>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct LiftoverIndexedEntry {
    range: Range<u64>,
    max: u64,
    data: ChainRange,
}
impl LiftoverIndexed {
    pub fn c_size(&self, c: &str) -> u64 {
        self.chromosomes.get(c).unwrap().1
    }
    fn from_liftover(liftover: &Liftover) -> Self {
        let mut chromosomes: BTreeMap<String, (Vec<LiftoverIndexedEntry>, u64)> = BTreeMap::new();

        for (mut from, mut to) in liftover.iter_ranges() {
            assert!(!from.range.is_empty(), "{from:?}");
            assert!(!to.range.is_empty(), "{to:?}");

            if from.range.orientation != SequenceOrientation::Forward {
                from.range = from.range.flip_orientation(from.size);
                to.range = to.range.flip_orientation(to.size);
            }

            let (chr, _size) = chromosomes
                .entry(from.range.name)
                .or_insert((vec![], from.size));

            chr.push(LiftoverIndexedEntry {
                range: from.range.at,
                max: 0,
                data: to,
            });
        }

        for (entries, _) in chromosomes.values_mut() {
            entries.sort_unstable_by_key(|e| (e.range.start, e.range.end));
            if !entries.is_empty() {
                entries[0].max = entries[0].range.end;
            }
            for i in 1..entries.len() {
                entries[i].max = cmp::max(entries[i - 1].max, entries[i].range.end);
            }
        }

        Self { chromosomes }
    }

    pub fn map(&self, loc: GenomePosition) -> impl Iterator<Item = GenomePosition> + use<'_> {
        self.map_raw(loc).map(|(r, _)| r)
    }
    pub fn map_range(&self, from: GenomeRange) -> impl Iterator<Item = GenomeRange> + use<'_> {
        self.map_range_raw(from).map(|(r, _)| r)
    }

    pub fn map_raw(
        &self,
        mut loc: GenomePosition,
    ) -> impl Iterator<Item = (GenomePosition, bool)> + use<'_> {
        let Some((ranges, size)) = self.chromosomes.get(&loc.name) else {
            return None.into_iter().flatten();
        };

        let original_orientation = loc.orientation;
        let flip_to_map = loc.orientation != SequenceOrientation::Forward;
        loc.set_orientation(SequenceOrientation::Forward, *size);
        let at = loc.at;
        drop(loc);

        // Note: `partition_point` splits by [true, true, true,|false, false]

        let ranges = {
            // We need `at < max`, so this is an lower bound (later ranges are still possible).
            #[allow(clippy::nonminimal_bool)]
            let lower_bound = ranges.partition_point(|e| !(at < e.max));
            &ranges[lower_bound..] // Select slice for which `!!(at < e.max)` or `at < e.max` holds.
        };
        let ranges = {
            // We need `start <= at`, so this is an upper bound (earlier ranges are still possible).
            let upper_bound = ranges.partition_point(|e| e.range.start <= at);
            &ranges[..upper_bound] // Select the slice for which `e.range.start <= at` holds.
        };

        Some(ranges.iter().filter_map(move |r| {
            if r.range.contains(&at) {
                let shift = at - r.range.start;
                let mut new = GenomePosition {
                    name: r.data.range.name.clone(),
                    at: r.data.range.at.start + shift,
                    orientation: r.data.range.orientation,
                };
                let flipped = if flip_to_map {
                    // If we flipped before, and do not need to flip again,
                    // we are already on the original strand, but the original reference sequence
                    // for this region is on the opposite strand, so should be fixed.
                    new.orientation == original_orientation
                } else {
                    // If we ended up on the opposite strand, let's make a note that we are flipping back.
                    new.orientation != original_orientation
                };
                new.set_orientation(original_orientation, r.data.size);
                Some((new, flipped))
            } else {
                None
            }
        }))
        .into_iter()
        .flatten()
    }
    pub fn map_range_raw(
        &self,
        mut from: GenomeRange,
    ) -> impl Iterator<Item = (GenomeRange, bool)> + use<'_> {
        let Some((ranges, size)) = self.chromosomes.get(&from.name) else {
            return None.into_iter().flatten();
        };

        let original_orientation = from.orientation;
        let flip_to_map = from.orientation != SequenceOrientation::Forward;
        from.set_orientation(SequenceOrientation::Forward, *size);
        let at = from.at.clone();
        drop(from);

        // Note: `partition_point` splits by [true, true, true,|false, false]

        let ranges = {
            // We need `at.start < max`, so this is an lower bound (later ranges are still possible).
            #[allow(clippy::nonminimal_bool)]
            let lower_bound = ranges.partition_point(|e| !(at.start < e.max));
            &ranges[lower_bound..] // Select slice for which `!!(at.start < e.max)` or `at.start < e.max` holds.
        };
        let ranges = {
            // We need `start <= at.end`, so this is an upper bound (earlier ranges are still possible).
            let upper_bound = ranges.partition_point(|e| e.range.start <= at.end);
            &ranges[..upper_bound] // Select the slice for which `e.range.start <= at.end` holds.
        };

        Some(ranges.iter().filter_map(move |r| {
            let intersected = r.range.clone().intersection(at.clone());
            if !intersected.is_empty() {
                let shift = intersected.start - r.range.start;
                let mut new = GenomeRange {
                    name: r.data.range.name.clone(),
                    at: (r.data.range.at.start + shift)
                        ..(r.data.range.at.start + shift + intersected.range_len()),
                    orientation: r.data.range.orientation,
                };
                let flipped = if flip_to_map {
                    // If we flipped before, and do not need to flip again,
                    // we are already on the original strand, but the original reference sequence
                    // for this region is on the opposite strand, so should be fixed.
                    new.orientation == original_orientation
                } else {
                    // If we ended up on the opposite strand, let's make a note that we are flipping back.
                    new.orientation != original_orientation
                };
                new.set_orientation(original_orientation, r.data.size);
                Some((new, flipped))
            } else {
                None
            }
        }))
        .into_iter()
        .flatten()
    }
}

mod boilerplate {
    use std::fmt;

    use biocore::location::SequenceOrientation;

    use super::{AlignmentBlock, Chain, ChainHeader, Liftover};

    impl fmt::Display for Liftover {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for chain in &self.chains {
                writeln!(f, "{chain}\n")?;
            }
            Ok(())
        }
    }
    impl fmt::Display for Chain {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                header,
                blocks,
                last_block,
            } = self;
            writeln!(f, "{header}")?;
            for block in blocks {
                writeln!(f, "{block}")?;
            }
            writeln!(f, "{last_block}")?;

            Ok(())
        }
    }
    impl fmt::Display for ChainHeader {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fn o(o: SequenceOrientation) -> &'static str {
                match o {
                    SequenceOrientation::Forward => "+",
                    SequenceOrientation::Reverse => "-",
                }
            }
            write!(
                f,
                "chain {} {} {} {} {} {} {} {} {} {} {} {}",
                self.score,
                self.t.range.name,
                self.t.size,
                o(self.t.range.orientation),
                self.t.range.at.start,
                self.t.range.at.end,
                self.q.range.name,
                self.q.size,
                o(self.q.range.orientation),
                self.q.range.at.start,
                self.q.range.at.end,
                self.id
            )
        }
    }
    impl fmt::Display for AlignmentBlock {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}\t{}\t{}", self.size, self.dt, self.dq)
        }
    }

    struct DebugPrint<T>(T);
    impl Liftover {
        pub fn to_debug_display(&self) -> String {
            format!("{:?}", DebugPrint(self))
        }
    }
    impl Chain {
        pub fn to_debug_display(&self) -> String {
            format!("{:?}", DebugPrint(self))
        }
    }
    impl std::fmt::Debug for DebugPrint<&Liftover> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for chain in &self.0.chains {
                writeln!(f, "{:?}\n", DebugPrint(chain))?;
            }
            Ok(())
        }
    }
    impl std::fmt::Debug for DebugPrint<&Chain> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Chain {
                header,
                blocks,
                last_block,
            } = &self.0;

            writeln!(f, "{header}")?;

            let mut t_start = header.t.range.at.start;
            let mut q_start = header.q.range.at.start;
            for block in blocks {
                let AlignmentBlock { size, dt, dq } = *block;
                write!(f, "\t{block}\t\t|\t\t{t_start} {q_start} -> ")?;

                t_start += size;
                q_start += size;

                t_start = t_start.strict_add_signed(dt);
                q_start += dq;

                writeln!(f, "{t_start} {q_start}")?;
            }

            writeln!(f, "{last_block}               {t_start} {q_start}")?;

            Ok(())
        }
    }
}
