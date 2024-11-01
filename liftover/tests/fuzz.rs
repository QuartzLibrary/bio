#![feature(iterator_try_collect)]
#![feature(strict_overflow_ops)]

#[path = "fuzz/internal.rs"]
mod internal;
#[path = "fuzz/testpoints.rs"]
mod testpoints;
#[path = "fuzz/ucsc.rs"]
mod ucsc;

use std::path::PathBuf;

use biocore::location::GenomeRange;
use utile::cache::Cache;

use liftover::{
    sources::{EnsemblHG, UcscHG},
    Liftover,
};

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
fn liftover_command() -> PathBuf {
    const LOCAL_LIFTOVER_CMD: &str = "tests/liftOver";
    PathBuf::from(CARGO_MANIFEST_DIR).join(LOCAL_LIFTOVER_CMD)
}
fn cache(prefix: &str) -> Cache {
    const LOCAL_DATA_DIR: &str = "tests/data";
    Cache::new(
        PathBuf::from(CARGO_MANIFEST_DIR).join(LOCAL_DATA_DIR),
        prefix,
    )
}

/// Ensures all available chain files are cached for testing.
///
/// NOTE: We use the global cache to avoid re-distributing the files in the repo.
#[ignore]
#[tokio::test]
async fn cache_chain_files() -> anyhow::Result<()> {
    // TODO: should probably store and check the hashes.

    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");

        let liftover = Liftover::load_human_ucsc(from, to).await.unwrap();

        drop(liftover);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");

        let liftover = Liftover::load_human_ensembl(from, to).await.unwrap();

        drop(liftover);
    }

    Ok(())
}

#[tokio::test]
async fn check_against_ucsc() -> anyhow::Result<()> {
    fn is_subset_of(a: &[GenomeRange], b: &[GenomeRange]) -> bool {
        const DUMMY_SIZE: u64 = 0; // It doesn't matter because they have the same orientation.
        a.iter()
            .all(|r| b.iter().any(|r2| r2.contains_range(r, DUMMY_SIZE)))
    }
    #[track_caller]
    fn assert_range(internal: &[GenomeRange], ucsc: &[GenomeRange]) {
        // They are not quite the same, because the liftover tool also merges some ranges,
        // so here we check that all the mapped values are a subset of the liftover tool.

        // Since ucsc sometimes merges the ranges, it should have <= fragments.
        assert!(ucsc.len() <= internal.len());

        if internal.len() <= 1 {
            // If we have one or fewer fragments, no merging can happen.
            assert_eq!(internal, ucsc);
        }
        assert!(is_subset_of(internal, ucsc));
        assert!(ucsc
            .iter()
            .all(|u| internal.iter().any(|i| i.at.start == u.at.start)));
        assert!(ucsc
            .iter()
            .all(|u| internal.iter().any(|i| i.at.end == u.at.end)));
    }

    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let key = &UcscHG::key(from, to);
        let entry = UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = internal::run_snps(liftover, snps);
        let ranges_internal = internal::run_ranges(liftover, ranges.clone());

        // let snps_ucsc = ucsc::run_snps(entry, &snps).await;
        // let ranges_ucsc = ucsc::run_ranges(entry, &ranges).await;

        let (snps_ucsc, ranges_ucsc) = ucsc::cache::get("ucsc", key);

        assert!(snps_internal == snps_ucsc);

        assert_eq!(ranges_ucsc.len(), ranges.len());

        for (internal, ucsc) in ranges_internal.iter().zip(&ranges_ucsc) {
            assert_range(internal, ucsc);
        }
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let key = &EnsemblHG::key(from, to);
        let entry = EnsemblHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = internal::run_snps(liftover, snps);
        let ranges_internal = internal::run_ranges(liftover, ranges.clone());

        // let snps_ucsc = ucsc::run_snps(entry, &snps).await;
        // let ranges_ucsc = ucsc::run_ranges(entry, &ranges).await;

        let (snps_ucsc, ranges_ucsc) = ucsc::cache::get("ensembl", key);

        assert!(snps_internal == snps_ucsc);

        assert_eq!(ranges_ucsc.len(), ranges.len());

        for (internal, ucsc) in ranges_internal.iter().zip(&ranges_ucsc) {
            assert_range(internal, ucsc);
        }
    }

    Ok(())
}
