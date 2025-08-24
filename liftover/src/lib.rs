#![feature(strict_overflow_ops)]
#![feature(iterator_try_collect)]
#![feature(impl_trait_in_assoc_type)]
#![feature(btree_set_entry)]

mod parse;

pub mod bindings;
pub mod sources;

use std::{cmp, collections::BTreeMap, ops::Range};

use utile::range::{RangeExt, RangeLen};

use biocore::{
    genome::{ArcContig, Contig},
    location::{
        orientation::{SequenceOrientation, WithOrientation},
        ContigPosition, ContigRange,
    },
};

/// https://genome.ucsc.edu/goldenPath/help/chain.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Liftover<From = ArcContig, To = ArcContig> {
    pub chains: Vec<Chain<From, To>>,

    // We stash a mapping so that we can upgrade other contig types transparently for conveneince.
    contigs: BTreeMap<String, From>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain<From = ArcContig, To = ArcContig> {
    pub header: ChainHeader<From, To>,
    blocks: Vec<AlignmentBlock>,
    last_block: u64,
}
/// The initial header line starts with the keyword `chain`,
/// followed by 11 required attribute values, and ends with a blank line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainHeader<From, To> {
    /// Chain score
    pub score: u64,
    /// (reference/target sequence)
    pub t: ChainRange<From>,
    /// (query sequence)
    pub q: ChainRange<To>,
    /// chain ID
    pub id: u32,
}
pub type ChainRange<C> = WithOrientation<ContigRange<C>>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlignmentBlock {
    /// the size of the ungapped alignment
    pub size: u64,
    /// the difference between the end of this block and the beginning of the next block (reference/target sequence)
    pub dt: i64,
    /// the difference between the end of this block and the beginning of the next block (query sequence)
    pub dq: u64,
}

impl<From, To> Liftover<From, To> {
    pub fn indexed(&self) -> LiftoverIndexed<From, To>
    where
        From: Contig + Ord + Clone,
        To: Contig + Clone,
    {
        LiftoverIndexed::from_liftover(self)
    }

    pub fn find_input_contig(&self, contig: impl AsRef<str>) -> Option<From>
    where
        From: Clone,
    {
        self.contigs.get(contig.as_ref()).cloned()
    }

    pub fn upgrade_contigs<NewFrom, NewTo>(
        self,
        mut from: impl FnMut(From) -> NewFrom,
        mut to: impl FnMut(To) -> NewTo,
    ) -> Liftover<NewFrom, NewTo> {
        let chains = self
            .chains
            .into_iter()
            .map(|c| c.upgrade_contigs(&mut from, &mut to))
            .collect();
        let contigs = self
            .contigs
            .into_iter()
            .map(|(k, v)| (k, from(v)))
            .collect();
        Liftover { chains, contigs }
    }
}
impl<From, To> Liftover<From, To>
where
    From: Contig,
    To: Contig + Clone,
{
    pub fn map<C: AsRef<str>>(
        &self,
        loc: ContigPosition<C>,
    ) -> impl Iterator<Item = ContigPosition<To>> + use<'_, From, To, C>
    where
        From: Clone,
    {
        let Some(internal) = self.find_input_contig(&loc.contig) else {
            log::warn!("[Liftover] Unknown contig: {}", loc.contig.as_ref());
            return None.into_iter().flatten();
        };

        let loc = WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigPosition {
                contig: internal.clone(),
                at: loc.at,
            },
        };

        Some(self.map_raw(loc).map(|mut r| {
            r.set_orientation(SequenceOrientation::Forward);
            r.v
        }))
        .into_iter()
        .flatten()
    }
    pub fn map_range<C: AsRef<str>>(
        &self,
        range: ContigRange<C>,
    ) -> impl Iterator<Item = ContigRange<To>> + use<'_, From, To, C>
    where
        From: Clone,
    {
        let Some(internal) = self.find_input_contig(&range.contig) else {
            log::warn!("[Liftover] Unknown contig: {}", range.contig.as_ref());
            return None.into_iter().flatten();
        };

        let range = WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigRange {
                contig: internal.clone(),
                at: range.at,
            },
        };

        Some(self.map_range_raw(range).map(|mut r| {
            r.set_orientation(SequenceOrientation::Forward);
            r.v
        }))
        .into_iter()
        .flatten()
    }

    pub fn map_raw(
        &self,
        loc: WithOrientation<ContigPosition<From>>,
    ) -> impl Iterator<Item = WithOrientation<ContigPosition<To>>> + use<'_, From, To> {
        self.chains.iter().flat_map(move |c| c.map_raw(&loc))
    }
    pub fn map_range_raw(
        &self,
        range: WithOrientation<ContigRange<From>>,
    ) -> impl Iterator<Item = WithOrientation<ContigRange<To>>> + use<'_, From, To> {
        self.chains
            .iter()
            .flat_map(move |c| c.map_range_raw(&range))
    }
}
impl<From, To> Chain<From, To>
where
    From: Contig,
    To: Contig + Clone,
{
    pub fn map_raw(
        &self,
        loc: &WithOrientation<ContigPosition<From>>,
    ) -> impl Iterator<Item = WithOrientation<ContigPosition<To>>> + use<'_, From, To> {
        let mut loc = loc.as_ref_contig();
        let initially_flipped = loc.orientation != self.header.t.orientation;
        loc.set_orientation(self.header.t.orientation);
        if !self.header.t.as_ref_contig().contains(&loc) {
            return None.into_iter().flatten();
        }
        let at = loc.v.at;
        #[expect(unused_variables)]
        let loc = ();

        let mut t_start = self.header.t.v.at.start;
        let mut q_start = self.header.q.v.at.start;

        let mapped = self
            .blocks()
            .filter_map(move |b| {
                let mut r = None;

                if (t_start..(t_start + b.size)).contains(&at) {
                    let new = WithOrientation {
                        orientation: self.header.q.orientation,
                        v: ContigPosition {
                            contig: self.header.q.v.contig.clone(),
                            at: q_start + (at - t_start),
                        },
                    };

                    r = Some(new);
                }

                t_start += b.size;
                q_start += b.size;

                t_start = t_start.strict_add_signed(b.dt);
                q_start += b.dq;

                r
            })
            .map(move |r| {
                if initially_flipped {
                    r.flip_orientation()
                } else {
                    r
                }
            });

        Some(mapped).into_iter().flatten()
    }
    pub fn map_range_raw(
        &self,
        range: &WithOrientation<ContigRange<From>>,
    ) -> impl Iterator<Item = WithOrientation<ContigRange<To>>> + use<'_, From, To> {
        let mut range = range.as_ref_contig();
        let initially_flipped = range.orientation != self.header.t.orientation;
        range.set_orientation(self.header.t.orientation);
        if !self.header.t.as_ref_contig().overlaps(&range) {
            return None.into_iter().flatten();
        }
        let at = range.v.at.clone();
        #[expect(unused_variables)]
        let range = ();

        let mut t_start = self.header.t.v.at.start;
        let mut q_start = self.header.q.v.at.start;

        let mapped = self
            .blocks()
            .filter_map(move |b| {
                let mut r = None;

                let intersected = (t_start..(t_start + b.size)).intersection(at.clone());

                if !intersected.is_empty() {
                    let shift = intersected.start - t_start;
                    r = Some(WithOrientation {
                        orientation: self.header.q.orientation,
                        v: ContigRange {
                            contig: self.header.q.v.contig.clone(),
                            at: (q_start + shift)..(q_start + shift + intersected.range_len()),
                        },
                    });
                }

                t_start += b.size;
                q_start += b.size;

                t_start = t_start.strict_add_signed(b.dt);
                q_start += b.dq;

                r
            })
            .map(move |r| {
                if initially_flipped {
                    r.flip_orientation()
                } else {
                    r
                }
            });

        Some(mapped).into_iter().flatten()
    }
}
impl<From, To> Chain<From, To> {
    pub fn blocks(&self) -> impl Iterator<Item = AlignmentBlock> + use<'_, From, To> {
        self.blocks.iter().copied().chain([AlignmentBlock {
            size: self.last_block,
            dt: 0,
            dq: 0,
        }])
    }

    pub fn upgrade_contigs<NewFrom, NewTo>(
        self,
        from: impl FnMut(From) -> NewFrom,
        to: impl FnMut(To) -> NewTo,
    ) -> Chain<NewFrom, NewTo> {
        Chain {
            header: self.header.upgrade_contigs(from, to),
            blocks: self.blocks,
            last_block: self.last_block,
        }
    }
}
impl<From, To> ChainHeader<From, To> {
    pub fn upgrade_contigs<NewFrom, NewTo>(
        self,
        from: impl FnMut(From) -> NewFrom,
        to: impl FnMut(To) -> NewTo,
    ) -> ChainHeader<NewFrom, NewTo> {
        ChainHeader {
            score: self.score,
            t: self.t.map_value(|v| v.map_contig(from)),
            q: self.q.map_value(|v| v.map_contig(to)),
            id: self.id,
        }
    }
}

impl<From, To> Liftover<From, To> {
    fn iter_ranges(
        &self,
    ) -> impl Iterator<Item = (ChainRange<From>, ChainRange<To>)> + use<'_, From, To>
    where
        From: Clone,
        To: Clone,
    {
        self.chains.iter().flat_map(|c| c.iter_ranges())
    }
}
impl<From, To> Chain<From, To> {
    fn iter_ranges(
        &self,
    ) -> impl Iterator<Item = (ChainRange<From>, ChainRange<To>)> + use<'_, From, To>
    where
        From: Clone,
        To: Clone,
    {
        let mut t_start = self.header.t.v.at.start;
        let mut q_start = self.header.q.v.at.start;

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
                WithOrientation {
                    orientation: self.header.t.orientation,
                    v: ContigRange {
                        contig: self.header.t.v.contig.clone(),
                        at: t_fragment,
                    },
                },
                WithOrientation {
                    orientation: self.header.q.orientation,
                    v: ContigRange {
                        contig: self.header.q.v.contig.clone(),
                        at: q_fragment,
                    },
                },
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftoverIndexed<In = ArcContig, Out = ArcContig> {
    chromosomes: BTreeMap<In, Vec<LiftoverIndexedEntry<Out>>>,
    contigs: BTreeMap<String, In>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct LiftoverIndexedEntry<Out> {
    range: Range<u64>,
    max: u64,
    data: ChainRange<Out>,
}
impl<From, To> LiftoverIndexed<From, To> {
    fn from_liftover(liftover: &Liftover<From, To>) -> Self
    where
        From: Contig + Ord + Clone,
        To: Contig + Clone,
    {
        let mut chromosomes: BTreeMap<From, Vec<LiftoverIndexedEntry<To>>> = BTreeMap::new();
        let mut contigs: BTreeMap<String, From> = BTreeMap::new();

        for (mut from, mut to) in liftover.iter_ranges() {
            assert!(!from.v.is_empty());
            assert!(!to.v.is_empty());

            if from.orientation != SequenceOrientation::Forward {
                from = from.flip_orientation();
                to = to.flip_orientation();
            }

            let chr = chromosomes.entry(from.v.contig.clone()).or_default();

            chr.push(LiftoverIndexedEntry {
                range: from.v.at,
                max: 0,
                data: to,
            });

            contigs.insert(from.v.contig.as_ref().to_owned(), from.v.contig.clone());
        }

        for entries in chromosomes.values_mut() {
            entries.sort_unstable_by_key(|e| (e.range.start, e.range.end));
            if !entries.is_empty() {
                entries[0].max = entries[0].range.end;
            }
            for i in 1..entries.len() {
                entries[i].max = cmp::max(entries[i - 1].max, entries[i].range.end);
            }
        }

        Self {
            chromosomes,
            contigs,
        }
    }

    pub fn find_input_contig(&self, contig: impl AsRef<str>) -> Option<From>
    where
        From: Clone,
    {
        self.contigs.get(contig.as_ref()).cloned()
    }
}
impl<From, To> LiftoverIndexed<From, To>
where
    From: Contig + Ord,
    To: Contig + Clone,
{
    pub fn map<C: AsRef<str>>(
        &self,
        loc: ContigPosition<C>,
    ) -> impl Iterator<Item = ContigPosition<To>> + use<'_, From, To, C>
    where
        From: Clone,
    {
        let Some(internal) = self.find_input_contig(&loc.contig) else {
            log::warn!("[Liftover] Unknown contig: {}", loc.contig.as_ref());
            return None.into_iter().flatten();
        };

        let range = WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigPosition {
                contig: internal.clone(),
                at: loc.at,
            },
        };

        Some(self.map_raw(&range).map(|mut r| {
            r.set_orientation(SequenceOrientation::Forward);
            r.v
        }))
        .into_iter()
        .flatten()
    }
    pub fn map_range<C: AsRef<str>>(
        &self,
        range: ContigRange<C>,
    ) -> impl Iterator<Item = ContigRange<To>> + use<'_, From, To, C>
    where
        From: Clone,
    {
        let Some(internal) = self.find_input_contig(&range.contig) else {
            log::warn!("[Liftover] Unknown contig: {}", range.contig.as_ref());
            return None.into_iter().flatten();
        };

        let range = WithOrientation {
            orientation: SequenceOrientation::Forward,
            v: ContigRange {
                contig: internal.clone(),
                at: range.at,
            },
        };

        Some(self.map_range_raw(&range).map(|mut r| {
            r.set_orientation(SequenceOrientation::Forward);
            r.v
        }))
        .into_iter()
        .flatten()
    }

    pub fn map_raw(
        &self,
        loc: &WithOrientation<ContigPosition<From>>,
    ) -> impl Iterator<Item = WithOrientation<ContigPosition<To>>> + use<'_, From, To> {
        let Some(ranges) = self.chromosomes.get(&loc.v.contig) else {
            return None.into_iter().flatten();
        };

        let mut loc = loc.as_ref_contig();
        let initially_flipped = loc.orientation != SequenceOrientation::Forward;
        loc.set_orientation(SequenceOrientation::Forward);
        let at = loc.v.at;
        #[expect(unused_variables)]
        let loc = ();

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
            if !r.range.contains(&at) {
                return None;
            }

            let shift = at - r.range.start;
            let new = WithOrientation {
                orientation: r.data.orientation,
                v: ContigPosition {
                    contig: r.data.v.contig.clone(),
                    at: r.data.v.at.start + shift,
                },
            };

            Some(if initially_flipped {
                new.flip_orientation()
            } else {
                new
            })
        }))
        .into_iter()
        .flatten()
    }
    pub fn map_range_raw(
        &self,
        from: &WithOrientation<ContigRange<From>>,
    ) -> impl Iterator<Item = WithOrientation<ContigRange<To>>> + use<'_, From, To> {
        let Some(ranges) = self.chromosomes.get(&from.v.contig) else {
            return None.into_iter().flatten();
        };

        let mut from = from.as_ref_contig();
        let initially_flipped = from.orientation != SequenceOrientation::Forward;
        from.set_orientation(SequenceOrientation::Forward);
        let at = from.v.at.clone();
        #[expect(unused_variables)]
        let from = ();

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
            if intersected.is_empty() {
                return None;
            }

            let shift = intersected.start - r.range.start;
            let new = WithOrientation {
                orientation: r.data.orientation,
                v: ContigRange {
                    contig: r.data.v.contig.clone(),
                    at: (r.data.v.at.start + shift)
                        ..(r.data.v.at.start + shift + intersected.range_len()),
                },
            };
            Some(if initially_flipped {
                new.flip_orientation()
            } else {
                new
            })
        }))
        .into_iter()
        .flatten()
    }
}

mod boilerplate {
    use std::fmt;

    use biocore::{genome::Contig, location::orientation::SequenceOrientation};

    use super::{AlignmentBlock, Chain, ChainHeader, Liftover};

    impl<From, To> fmt::Display for Liftover<From, To>
    where
        From: Contig,
        To: Contig,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for chain in &self.chains {
                writeln!(f, "{chain}\n")?;
            }
            Ok(())
        }
    }
    impl<From, To> fmt::Display for Chain<From, To>
    where
        From: Contig,
        To: Contig,
    {
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
    impl<From, To> fmt::Display for ChainHeader<From, To>
    where
        From: Contig,
        To: Contig,
    {
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
                self.t.v.contig.as_ref(),
                self.t.v.contig.size(),
                o(self.t.orientation),
                self.t.v.at.start,
                self.t.v.at.end,
                self.q.v.contig.as_ref(),
                self.q.v.contig.size(),
                o(self.q.orientation),
                self.q.v.at.start,
                self.q.v.at.end,
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
    impl<From, To> Liftover<From, To>
    where
        From: Contig,
        To: Contig,
    {
        pub fn to_debug_display(&self) -> String {
            format!("{:?}", DebugPrint(self))
        }
    }
    impl<From, To> Chain<From, To>
    where
        From: Contig,
        To: Contig,
    {
        pub fn to_debug_display(&self) -> String {
            format!("{:?}", DebugPrint(self))
        }
    }
    impl<From, To> std::fmt::Debug for DebugPrint<&Liftover<From, To>>
    where
        From: Contig,
        To: Contig,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for chain in &self.0.chains {
                writeln!(f, "{:?}\n", DebugPrint(chain))?;
            }
            Ok(())
        }
    }
    impl<From, To> std::fmt::Debug for DebugPrint<&Chain<From, To>>
    where
        From: Contig,
        To: Contig,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Chain {
                header,
                blocks,
                last_block,
            } = &self.0;

            writeln!(f, "{header}")?;

            let mut t_start = header.t.v.at.start;
            let mut q_start = header.q.v.at.start;
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
