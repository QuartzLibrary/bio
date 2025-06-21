use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{collections::BTreeMap, ops::Range};

use biocore::{
    genome::Contig,
    location::{GenomePosition, GenomeRange},
};
use utile::resource::{RawResource, RawResourceExt};

use liftover::{
    sources::{EnsemblHG, EnsemblResource, UcscHG, UcscResource},
    Chain, Liftover,
};

/// Generates and caches the testpoints.
/// Mostly useful for diffing/debugging as we can compute these on the fly.
#[ignore]
#[test]
fn cache_testpoints() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();
        cache::store(&Liftover::load(entry).unwrap(), "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();
        cache::store(&Liftover::load(entry).unwrap(), "ensembl", &resource.key());
    }

    Ok(())
}
/// Checks the cached testpoints are still the latest ones.
#[test]
fn check_testpoints() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();
        cache::assert(&Liftover::load(entry).unwrap(), "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();
        cache::assert(&Liftover::load(entry).unwrap(), "ensembl", &resource.key());
    }

    Ok(())
}

/// Randomly generates testpoints for a given chain file.
/// Equivalent chain files will generate the same testpoints.
pub fn get(liftover: &Liftover) -> (Vec<GenomePosition>, Vec<GenomeRange>) {
    let snps = generate_snps(liftover, &mut SmallRng::seed_from_u64(42)); // For reproducibility.
    let ranges = generate_ranges(liftover, &mut SmallRng::seed_from_u64(42)); // For reproducibility.

    (snps, ranges)
}

fn generate_ranges(liftover: &Liftover, rng: &mut impl Rng) -> Vec<GenomeRange> {
    contigs(liftover)
        .into_iter()
        .flat_map(|(name, size)| {
            [0..1, 0..size, size - 1..size]
                .into_iter()
                .chain((0..100).map(|_| {
                    let from = rng.random_range(0..size);
                    let to = from + rng.random_range(1..1000);
                    let to = Ord::min(size, to);
                    from..to
                }))
                .map(|range| GenomeRange {
                    name: name.clone(),
                    at: range,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            liftover
                .chains
                .iter()
                .flat_map(|chain| generate_range_edge_cases(chain, rng)),
        )
        .collect()
}
fn generate_snps(liftover: &Liftover, rng: &mut impl Rng) -> Vec<GenomePosition> {
    contigs(liftover)
        .into_iter()
        .flat_map(|(name, size)| {
            [0, size - 1]
                .into_iter()
                .chain((0..100).map(|_| rng.random_range(0..size)))
                .map(|at| GenomePosition {
                    name: name.clone(),
                    at,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            liftover
                .chains
                .iter()
                .flat_map(|chain| generate_snp_edge_cases(chain, rng)),
        )
        .collect()
}

fn contigs(liftover: &Liftover) -> Vec<(String, u64)> {
    let mut contigs = BTreeMap::new();
    for chain in &liftover.chains {
        let contig = &chain.header.t;
        let old = contigs.insert(contig.v.name.as_ref().to_owned(), contig.v.name.size());
        if let Some(old) = old {
            assert_eq!(old, contig.v.name.size());
        }
    }
    contigs.into_iter().collect()
}

fn generate_snp_edge_cases(chain: &Chain, rng: &mut impl Rng) -> Vec<GenomePosition> {
    let mut t_start = chain.header.t.v.at.start;

    let mut cases = vec![];

    for b in chain.blocks() {
        let weird = b.dt < 0;
        let peculiar = (b.size == 0 || b.dt == 0 || b.dq == 0) && rng.random_bool(1. / 100.);
        if weird || peculiar || rng.random_bool(1. / 1000.) {
            let new = [
                t_start,
                t_start + 1,
                t_start.strict_add_signed(b.dt),
                t_start.strict_add_signed(b.dt) + 1,
            ];
            cases.extend(new);

            if t_start.saturating_add_signed(b.dt) > 0 {
                cases.push(t_start.strict_add_signed(b.dt) - 1);
            }

            if t_start != 0 {
                cases.push(t_start - 1);
            }
        }

        t_start += b.size;

        t_start = t_start.strict_add_signed(b.dt);
    }

    cases
        .into_iter()
        .map(|at| GenomePosition {
            name: chain.header.t.v.name.as_ref().to_owned(),
            at,
        })
        .collect()
}
fn generate_range_edge_cases(chain: &Chain, rng: &mut impl Rng) -> Vec<GenomeRange> {
    let mut t_start = chain.header.t.v.at.start;

    let mut cases = vec![];

    for b in chain.blocks() {
        let weird = b.dt < 0;
        let peculiar = (b.size == 0 || b.dt == 0 || b.dq == 0) && rng.random_bool(1. / 100.);
        if weird || peculiar || rng.random_bool(1. / 1000.) {
            let new = [
                // t_start..t_start,
                t_start..t_start + 1,
                t_start..t_start + 2,
                t_start..t_start.strict_add_signed(b.dt) + 1,
                t_start..t_start.strict_add_signed(b.dt) + 2,
                // t_start.strict_add_signed(b.dt)..t_start.strict_add_signed(b.dt),
                t_start.strict_add_signed(b.dt)..t_start.strict_add_signed(b.dt) + 1,
                t_start.strict_add_signed(b.dt)..t_start.strict_add_signed(b.dt) + 2,
            ];
            cases.extend(new);

            if t_start.saturating_add_signed(b.dt) > 0 {
                let new = [
                    t_start.strict_add_signed(b.dt) - 1..t_start.strict_add_signed(b.dt),
                    t_start.strict_add_signed(b.dt) - 1..t_start.strict_add_signed(b.dt) + 1,
                    t_start.strict_add_signed(b.dt) - 1..t_start.strict_add_signed(b.dt) + 2,
                ];
                cases.extend(new);
            }

            if t_start != 0 {
                let new = [
                    t_start - 1..t_start,
                    t_start - 1..t_start + 1,
                    t_start - 1..t_start + 2,
                    t_start - 1..t_start.strict_add_signed(b.dt),
                    t_start - 1..t_start.strict_add_signed(b.dt) + 2,
                ];

                cases.extend(new);
            }
        }

        t_start += b.size;

        t_start = t_start.strict_add_signed(b.dt);
    }

    cases
        .into_iter()
        .map(|Range { start, end }| {
            let at = if end < start { end..start } else { start..end };
            // assert!(!at.is_empty());
            GenomeRange {
                name: chain.header.t.v.name.as_ref().to_owned(),
                at,
            }
        })
        .collect()
}

/// Mostly useful for diffing/debugging as we can compute these on the fly.
pub mod cache {
    use std::path::PathBuf;

    use liftover::Liftover;
    use utile::{cache::FsCacheEntry, resource::RawResourceExt};

    pub fn store(liftover: &Liftover, prefix: &str, key: &str) {
        let (snps, ranges) = super::get(liftover);

        snp_testpoints(prefix, key).write_json_lines(snps).unwrap();
        range_testpoints(prefix, key)
            .write_json_lines(ranges)
            .unwrap();
    }
    pub fn assert(liftover: &Liftover, prefix: &str, key: &str) {
        let (snps, ranges) = super::get(liftover);

        assert!(
            snps == snp_testpoints(prefix, key)
                .read_json_lines()
                .unwrap()
                .try_collect::<Vec<_>>()
                .unwrap()
        );
        assert!(
            ranges
                == range_testpoints(prefix, key)
                    .read_json_lines()
                    .unwrap()
                    .try_collect::<Vec<_>>()
                    .unwrap()
        );
    }

    fn snp_testpoints(prefix: &str, key: &str) -> FsCacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("snp_testpoints.jsonl"))
    }
    fn range_testpoints(prefix: &str, key: &str) -> FsCacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("range_testpoints.jsonl"))
    }
}
