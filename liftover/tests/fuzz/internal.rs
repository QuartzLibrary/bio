use biocore::location::{ContigPosition, ContigRange};

use liftover::{
    Liftover, LiftoverIndexed,
    sources::{EnsemblHG, EnsemblResource, UcscHG, UcscResource},
};
use resource::{RawResource, RawResourceExt};

#[ignore]
#[test]
fn cache_testpoints_internal() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::store(snps_internal, ranges_internal, "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::store(snps_internal, ranges_internal, "ensembl", &resource.key());
    }

    Ok(())
}
#[test]
fn check_testpoints_internal() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = run_snps(liftover, snps);
        let ranges_internal = run_ranges(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ensembl", &resource.key());
    }

    Ok(())
}
#[test]
#[ignore = "slow"]
fn check_testpoints_slow() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_internal = run_snps_slow(liftover, snps);
        let ranges_internal = run_ranges_slow(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_internal = run_snps_slow(liftover, snps);
        let ranges_internal = run_ranges_slow(liftover, ranges);

        cache::assert(snps_internal, ranges_internal, "ensembl", &resource.key());
    }

    Ok(())
}

pub fn run_snps(liftover: &LiftoverIndexed, snps: Vec<ContigPosition>) -> Vec<Vec<ContigPosition>> {
    snps.into_iter()
        .map(|l| {
            liftover
                .map(l)
                .map(|l| l.map_contig(|c| c.as_ref().to_owned()))
                .collect()
        })
        .collect()
}
pub fn run_ranges(liftover: &LiftoverIndexed, ranges: Vec<ContigRange>) -> Vec<Vec<ContigRange>> {
    ranges
        .clone()
        .into_iter()
        .map(|r| {
            liftover
                .map_range(r)
                .map(|r| r.map_contig(|c| c.as_ref().to_owned()))
                .collect()
        })
        .collect()
}

pub fn run_snps_slow(liftover: &Liftover, snps: Vec<ContigPosition>) -> Vec<Vec<ContigPosition>> {
    snps.into_iter()
        .map(|l| {
            liftover
                .map(l)
                .map(|l| l.map_contig(|c| c.as_ref().to_owned()))
                .collect()
        })
        .collect()
}
pub fn run_ranges_slow(liftover: &Liftover, ranges: Vec<ContigRange>) -> Vec<Vec<ContigRange>> {
    ranges
        .clone()
        .into_iter()
        .map(|r| {
            liftover
                .map_range(r)
                .map(|r| r.map_contig(|c| c.as_ref().to_owned()))
                .collect()
        })
        .collect()
}

/// Mostly useful for diffing/debugging as we can compute these on the fly.
pub mod cache {
    use std::path::PathBuf;

    use biocore::location::{ContigPosition, ContigRange};
    use resource::{RawResourceExt, fs::FsCacheEntry};

    pub fn store(
        snps_internal: Vec<Vec<ContigPosition>>,
        ranges_internal: Vec<Vec<ContigRange>>,
        prefix: &str,
        key: &str,
    ) {
        snp_testpoints_internal_target(prefix, key)
            .write_json_lines(snps_internal)
            .unwrap();
        range_testpoints_internal_target(prefix, key)
            .write_json_lines(ranges_internal)
            .unwrap();
    }
    pub fn assert(
        snps_internal: Vec<Vec<ContigPosition>>,
        ranges_internal: Vec<Vec<ContigRange>>,
        prefix: &str,
        key: &str,
    ) {
        let snps_target: Vec<_> = snp_testpoints_internal_target(prefix, key)
            .read_json_lines::<Vec<ContigPosition>>()
            .unwrap()
            .map(|l| l.unwrap())
            .collect();
        assert_eq!(snps_internal.len(), snps_target.len());
        for (mut snps_internal, mut snp_target) in snps_internal.into_iter().zip(snps_target) {
            snps_internal.sort();
            snp_target.sort();
            assert_eq!(snps_internal, snp_target);
        }

        let ranges_target: Vec<_> = range_testpoints_internal_target(prefix, key)
            .read_json_lines::<Vec<ContigRange>>()
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(ranges_internal.len(), ranges_target.len());
        for (mut ranges_internal, mut range_target) in
            ranges_internal.into_iter().zip(ranges_target)
        {
            ranges_internal.sort();
            range_target.sort();
            assert!(ranges_internal == range_target);
        }
    }

    fn snp_testpoints_internal_target(prefix: &str, key: &str) -> FsCacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("snp_testpoints_internal.jsonl"))
    }
    fn range_testpoints_internal_target(prefix: &str, key: &str) -> FsCacheEntry {
        super::super::cache(prefix)
            .entry(PathBuf::from(key).join("range_testpoints_internal.jsonl"))
    }
}
