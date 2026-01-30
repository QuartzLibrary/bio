#![feature(iterator_try_collect)]

#[path = "fuzz/internal.rs"]
mod internal;
#[path = "fuzz/testpoints.rs"]
mod testpoints;
#[path = "fuzz/ucsc.rs"]
mod ucsc;

use std::path::PathBuf;

use biocore::location::ContigRange;
use resource::{RawResource, RawResourceExt, fs::FsCache};

use liftover::{
    Liftover,
    sources::{EnsemblHG, EnsemblResource, UcscHG, UcscResource},
};

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
fn liftover_command() -> PathBuf {
    const LOCAL_LIFTOVER_CMD: &str = "tests/liftOver";
    PathBuf::from(CARGO_MANIFEST_DIR).join(LOCAL_LIFTOVER_CMD)
}
fn cache(prefix: &str) -> FsCache {
    const LOCAL_DATA_DIR: &str = "tests/data";
    FsCache::new(
        PathBuf::from(CARGO_MANIFEST_DIR)
            .join(LOCAL_DATA_DIR)
            .join(prefix),
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

        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = Liftover::load(entry).unwrap();

        drop(liftover);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");

        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = Liftover::load(entry).unwrap();

        drop(liftover);
    }

    Ok(())
}

#[test]
fn check_against_ucsc() -> anyhow::Result<()> {
    fn is_subset_of(a: &[ContigRange], b: &[ContigRange]) -> bool {
        a.iter().all(|r| b.iter().any(|r2| r2.contains_range(r)))
    }
    #[track_caller]
    fn assert_range(internal: &[ContigRange], ucsc: &[ContigRange]) {
        // They are not quite the same, because the liftover tool also merges some ranges,
        // so here we check that all the mapped values are a subset of the liftover tool.

        // Since ucsc sometimes merges the ranges, it should have <= fragments.
        assert!(ucsc.len() <= internal.len());

        if internal.len() <= 1 {
            // If we have one or fewer fragments, no merging can happen.
            assert_eq!(internal, ucsc);
        }
        assert!(is_subset_of(internal, ucsc));
        assert!(
            ucsc.iter()
                .all(|u| internal.iter().any(|i| i.at.start == u.at.start))
        );
        assert!(
            ucsc.iter()
                .all(|u| internal.iter().any(|i| i.at.end == u.at.end))
        );
    }

    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = internal::run_snps(liftover, snps);
        let ranges_internal = internal::run_ranges(liftover, ranges.clone());

        // let snps_ucsc = ucsc::run_snps(entry, &snps).await;
        // let ranges_ucsc = ucsc::run_ranges(entry, &ranges).await;

        let (snps_ucsc, ranges_ucsc) = ucsc::cache::get("ucsc", &resource.key());

        // assert!(snps_internal == snps_ucsc);
        for (mut internal, mut ucsc) in snps_internal.into_iter().zip(snps_ucsc) {
            internal.sort();
            ucsc.sort();
            ucsc.dedup();
            assert_eq!(internal, ucsc);
        }

        assert_eq!(ranges_ucsc.len(), ranges.len());

        for (mut internal, mut ucsc) in ranges_internal.into_iter().zip(ranges_ucsc) {
            internal.sort();
            ucsc.sort();
            ucsc.dedup();
            assert_range(&internal, &ucsc);
        }
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry).unwrap();

        let (snps, ranges) = testpoints::get(liftover);

        let liftover = &liftover.indexed();

        let snps_internal = internal::run_snps(liftover, snps);
        let ranges_internal = internal::run_ranges(liftover, ranges.clone());

        // let snps_ucsc = ucsc::run_snps(entry, &snps).await;
        // let ranges_ucsc = ucsc::run_ranges(entry, &ranges).await;

        let (snps_ucsc, ranges_ucsc) = ucsc::cache::get("ensembl", &resource.key());

        // assert!(snps_internal == snps_ucsc);
        for (mut internal, mut ucsc) in snps_internal.into_iter().zip(snps_ucsc) {
            internal.sort();
            ucsc.sort();
            ucsc.dedup();
            assert_eq!(internal, ucsc);
        }

        assert_eq!(ranges_ucsc.len(), ranges.len());

        for (mut internal, mut ucsc) in ranges_internal.into_iter().zip(ranges_ucsc) {
            internal.sort();
            ucsc.sort();
            ucsc.dedup();
            assert_range(&internal, &ucsc);
        }
    }

    Ok(())
}
