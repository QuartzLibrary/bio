use biocore::location::{GenomePosition, GenomeRange};

use liftover::{
    sources::{EnsemblHG, UcscHG},
    Liftover, LiftoverIndexed,
};

#[ignore]
#[test]
fn cache_testpoints_internal() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let key = &UcscHG::key(from, to);
        let entry = UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::store(snps_internal, ranges_internal, "ucsc", key);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let key = &EnsemblHG::key(from, to);
        let entry = EnsemblHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::store(snps_internal, ranges_internal, "ensembl", key);
    }

    Ok(())
}
#[test]
fn check_testpoints_internal() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let key = &UcscHG::key(from, to);
        let entry = UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ucsc", key);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let key = &EnsemblHG::key(from, to);
        let entry = EnsemblHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ensembl", key);
    }

    Ok(())
}
#[test]
fn check_testpoints_slow() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let key = &UcscHG::key(from, to);
        let entry = UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_internal = run_snps_slow(liftover, snps);
        let ranges_internal = run_ranges_slow(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ucsc", key);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let key = &EnsemblHG::key(from, to);
        let entry = EnsemblHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_internal = run_snps_slow(liftover, snps);
        let ranges_internal = run_ranges_slow(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ensembl", key);
    }

    Ok(())
}

pub fn run_snps(liftover: &LiftoverIndexed, snps: Vec<GenomePosition>) -> Vec<Vec<GenomePosition>> {
    snps.into_iter().map(|l| liftover.map(l)).collect()
}
pub fn run_ranges(liftover: &LiftoverIndexed, ranges: Vec<GenomeRange>) -> Vec<Vec<GenomeRange>> {
    ranges
        .clone()
        .into_iter()
        .map(|r| liftover.map_range(r))
        .collect()
}

pub fn run_snps_slow(liftover: &Liftover, snps: Vec<GenomePosition>) -> Vec<Vec<GenomePosition>> {
    snps.into_iter().map(|l| liftover.map(l)).collect()
}
pub fn run_ranges_slow(liftover: &Liftover, ranges: Vec<GenomeRange>) -> Vec<Vec<GenomeRange>> {
    ranges
        .clone()
        .into_iter()
        .map(|r| liftover.map_range(r))
        .collect()
}

/// Mostly useful for diffing/debugging as we can compute these on the fly.
pub mod cache {
    use std::path::PathBuf;

    use biocore::location::{GenomePosition, GenomeRange};
    use utile::cache::CacheEntry;

    pub fn store(
        snps_internal: Vec<Vec<GenomePosition>>,
        ranges_internal: Vec<Vec<GenomeRange>>,
        prefix: &str,
        key: &str,
    ) {
        snp_testpoints_internal_target(prefix, key)
            .set_json_lines(snps_internal)
            .unwrap();
        range_testpoints_internal_target(prefix, key)
            .set_json_lines(ranges_internal)
            .unwrap();
    }
    pub fn assert(
        snps_internal: Vec<Vec<GenomePosition>>,
        ranges_internal: Vec<Vec<GenomeRange>>,
        prefix: &str,
        key: &str,
    ) {
        assert!(
            snps_internal
                == snp_testpoints_internal_target(prefix, key)
                    .get_as_json_lines::<Vec<GenomePosition>>()
                    .unwrap()
        );
        assert!(
            ranges_internal
                == range_testpoints_internal_target(prefix, key)
                    .get_as_json_lines::<Vec<GenomeRange>>()
                    .unwrap()
        );
    }

    fn snp_testpoints_internal_target(prefix: &str, key: &str) -> CacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("snp_testpoints_internal.jsonl"))
    }
    fn range_testpoints_internal_target(prefix: &str, key: &str) -> CacheEntry {
        super::super::cache(prefix)
            .entry(PathBuf::from(key).join("range_testpoints_internal.jsonl"))
    }
}
