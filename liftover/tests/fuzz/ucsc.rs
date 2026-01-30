use std::path::Path;

use biocore::location::{ContigPosition, ContigRange};

use liftover::{
    Liftover,
    bindings::{self, ucsc::UcscLiftoverSettings},
    sources::{EnsemblHG, EnsemblResource, UcscHG, UcscResource},
};
use resource::{RawResource, RawResourceExt};

#[ignore]
#[tokio::test]
async fn cache_testpoints_ucsc() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry.clone()).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry.cache().unwrap(), &snps).await;
        let ranges_ucsc = run_ranges(entry.cache().unwrap(), &ranges).await;

        cache::store(snps_ucsc, ranges_ucsc, "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry.clone()).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry.cache().unwrap(), &snps).await;
        let ranges_ucsc = run_ranges(entry.cache().unwrap(), &ranges).await;

        cache::store(snps_ucsc, ranges_ucsc, "ensembl", &resource.key());
    }

    Ok(())
}
#[ignore]
#[tokio::test]
async fn check_testpoints_ucsc() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry.clone()).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry.cache().unwrap(), &snps).await;
        let ranges_ucsc = run_ranges(entry.cache().unwrap(), &ranges).await;

        cache::assert(snps_ucsc, ranges_ucsc, "ucsc", &resource.key());
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry.clone()).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry.cache().unwrap(), &snps).await;
        let ranges_ucsc = run_ranges(entry.cache().unwrap(), &ranges).await;

        cache::assert(snps_ucsc, ranges_ucsc, "ensembl", &resource.key());
    }

    Ok(())
}
#[ignore]
#[tokio::test]
async fn check_testpoints_ucsc_web() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        if !UcscHG::is_available_in_online_interface(from, to) {
            continue;
        }

        println!("{from} {to}");
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = &Liftover::load(entry.clone()).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps_web(from, to, &snps).await;
        let ranges_ucsc = run_ranges_web(from, to, &ranges).await;

        cache::assert(snps_ucsc, ranges_ucsc, "ucsc", &resource.key());
    }

    Ok(())
}

pub async fn run_snps(
    liftover_path: impl AsRef<Path>,
    snps: &[ContigPosition],
) -> Vec<Vec<ContigPosition>> {
    bindings::ucsc::cli::liftover_human_snps(
        snps,
        liftover_path,
        super::liftover_command(),
        UcscLiftoverSettings::loose(),
    )
    .await
    .unwrap()
    .into_iter()
    .try_collect()
    .unwrap()
}
pub async fn run_ranges(
    liftover_path: impl AsRef<Path>,
    ranges: &[ContigRange],
) -> Vec<Vec<ContigRange>> {
    bindings::ucsc::cli::liftover_human(
        ranges,
        liftover_path,
        super::liftover_command(),
        UcscLiftoverSettings::loose(),
    )
    .await
    .unwrap()
    .into_iter()
    .try_collect()
    .unwrap()
}

pub async fn run_snps_web(
    from: UcscHG,
    to: UcscHG,
    snps: &[ContigPosition],
) -> Vec<Vec<ContigPosition>> {
    let client = reqwest::Client::new();

    bindings::ucsc::web::liftover_human_snps(&client, from, snps, to, UcscLiftoverSettings::loose())
        .await
        .unwrap()
        .into_iter()
        .try_collect()
        .unwrap()
}
pub async fn run_ranges_web(
    from: UcscHG,
    to: UcscHG,
    ranges: &[ContigRange],
) -> Vec<Vec<ContigRange>> {
    let client = reqwest::Client::new();

    bindings::ucsc::web::liftover_human(&client, from, ranges, to, UcscLiftoverSettings::loose())
        .await
        .unwrap()
        .into_iter()
        .try_collect()
        .unwrap()
}

/// Cached to keep the tests working without access to CLI tool.
pub mod cache {
    use std::path::PathBuf;

    use biocore::location::{ContigPosition, ContigRange};
    use resource::{RawResourceExt, fs::FsCacheEntry};

    pub fn get(prefix: &str, key: &str) -> (Vec<Vec<ContigPosition>>, Vec<Vec<ContigRange>>) {
        (
            snp_testpoints_ucsc_target(prefix, key)
                .read_json_lines()
                .unwrap()
                .try_collect::<Vec<_>>()
                .unwrap(),
            range_testpoints_ucsc_target(prefix, key)
                .read_json_lines()
                .unwrap()
                .try_collect::<Vec<_>>()
                .unwrap(),
        )
    }
    pub fn store(
        snps_internal: Vec<Vec<ContigPosition>>,
        ranges_internal: Vec<Vec<ContigRange>>,
        prefix: &str,
        key: &str,
    ) {
        snp_testpoints_ucsc_target(prefix, key)
            .write_json_lines(snps_internal)
            .unwrap();
        range_testpoints_ucsc_target(prefix, key)
            .write_json_lines(ranges_internal)
            .unwrap();
    }
    pub fn assert(
        snps_internal: Vec<Vec<ContigPosition>>,
        ranges_internal: Vec<Vec<ContigRange>>,
        prefix: &str,
        key: &str,
    ) {
        let (found_snps, found_ranges) = get(prefix, key);
        assert!(snps_internal == found_snps);
        assert!(ranges_internal == found_ranges);
    }

    fn snp_testpoints_ucsc_target(prefix: &str, key: &str) -> FsCacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("snp_testpoints_uscs.jsonl"))
    }
    fn range_testpoints_ucsc_target(prefix: &str, key: &str) -> FsCacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("range_testpoints_uscs.jsonl"))
    }
}
