#![feature(strict_overflow_ops)]
#![feature(iterator_try_collect)]

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
    pub fn map(&self, loc: GenomePosition) -> Vec<GenomePosition> {
        let mut matches: Vec<_> = self
            .chains
            .iter()
            .flat_map(|c| c.map(loc.clone()))
            .collect();
        matches.sort();
        matches.dedup();
        matches
    }
    pub fn map_range(&self, range: GenomeRange) -> Vec<GenomeRange> {
        let mut matches: Vec<_> = self
            .chains
            .iter()
            .flat_map(|c| c.map_range(range.clone()))
            .collect();
        matches.sort();
        matches.dedup();
        matches
    }
    fn iter_ranges(&self) -> impl Iterator<Item = (ChainRange, ChainRange)> + use<'_> {
        self.chains.iter().flat_map(|c| c.iter_ranges())
    }
}
impl Chain {
    pub fn map(&self, mut loc: GenomePosition) -> Vec<GenomePosition> {
        let original_orientation = loc.orientation;
        loc.set_orientation(self.header.t.range.orientation, self.header.t.size);
        if !self.header.t.contains(&loc) {
            return vec![];
        }
        let at = loc.at;
        drop(loc);

        let mut t_start = self.header.t.range.at.start;
        let mut q_start = self.header.q.range.at.start;

        let mut matches = vec![];

        for b in self.blocks() {
            if (t_start..(t_start + b.size)).contains(&at) {
                let new = GenomePosition {
                    name: self.header.q.range.name.clone(),
                    at: q_start + (at - t_start),
                    orientation: self.header.q.range.orientation,
                };

                matches.push(new);
            }

            t_start += b.size;
            q_start += b.size;

            t_start = t_start.strict_add_signed(b.dt);
            q_start += b.dq;
        }

        for r in &mut matches {
            r.set_orientation(original_orientation, self.header.q.size);
        }

        matches
    }
    pub fn map_range(&self, range: GenomeRange) -> Vec<GenomeRange> {
        let intersected = match self.header.t.intersect(range.clone()) {
            Some(intersected) if intersected.is_empty() => return vec![],
            Some(intersected) => intersected,
            None => return vec![],
        };
        assert_eq!(intersected.name, range.name);
        let original_orientation = range.orientation;
        drop(range);

        let mut t_start = self.header.t.range.at.start;
        let mut q_start = self.header.q.range.at.start;

        let mut intersections = vec![];

        for b in self.blocks() {
            let intersected = (t_start..(t_start + b.size)).intersect(intersected.at.clone());

            if !intersected.is_empty() {
                let shift = intersected.start - t_start;
                intersections.push(GenomeRange {
                    name: self.header.q.range.name.clone(),
                    at: (q_start + shift)..(q_start + shift + intersected.range_len()),
                    orientation: self.header.q.range.orientation,
                });
            }

            t_start += b.size;
            q_start += b.size;

            t_start = t_start.strict_add_signed(b.dt);
            q_start += b.dq;
        }

        for r in &mut intersections {
            r.set_orientation(original_orientation, self.header.q.size);
        }

        intersections
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

        for (mut from, to) in liftover.iter_ranges() {
            assert!(!from.range.is_empty(), "{from:?}");
            assert!(!to.range.is_empty(), "{to:?}");

            from.range
                .set_orientation(SequenceOrientation::Forward, from.size);

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
    pub fn map(&self, mut loc: GenomePosition) -> Vec<GenomePosition> {
        let Some((ranges, size)) = self.chromosomes.get(&loc.name) else {
            return vec![];
        };

        let original_orientation = loc.orientation;
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

        let mut matches = vec![];

        for r in ranges {
            if r.range.contains(&at) {
                let shift = at - r.range.start;
                let mut new = GenomePosition {
                    name: r.data.range.name.clone(),
                    at: r.data.range.at.start + shift,
                    orientation: r.data.range.orientation,
                };
                new.set_orientation(original_orientation, r.data.size);
                matches.push(new);
            }
        }

        matches.sort();
        matches.dedup();

        matches
    }
    pub fn map_range(&self, mut from: GenomeRange) -> Vec<GenomeRange> {
        let Some((ranges, size)) = self.chromosomes.get(&from.name) else {
            return vec![];
        };

        let original_orientation = from.orientation;
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

        let mut intersections = vec![];

        for r in ranges {
            let intersected = r.range.clone().intersect(at.clone());
            if !intersected.is_empty() {
                let shift = intersected.start - r.range.start;
                let mut new = GenomeRange {
                    name: r.data.range.name.clone(),
                    at: (r.data.range.at.start + shift)
                        ..(r.data.range.at.start + shift + intersected.range_len()),
                    orientation: r.data.range.orientation,
                };
                new.set_orientation(original_orientation, r.data.size);
                intersections.push(new);
            }
        }

        intersections.sort();
        intersections.dedup();

        intersections
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
