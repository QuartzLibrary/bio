use std::path::Path;

use biocore::location::{GenomePosition, GenomeRange};

use liftover::{
    bindings::{self, ucsc::UcscLiftoverSettings},
    sources::{EnsemblHG, UcscHG},
    Liftover,
};

#[ignore]
#[tokio::test]
async fn cache_testpoints_ucsc() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let key = &UcscHG::key(from, to);
        let entry = &UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry, &snps).await;
        let ranges_ucsc = run_ranges(entry, &ranges).await;

        cache::store(snps_ucsc, ranges_ucsc, "ucsc", key);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let key = &EnsemblHG::key(from, to);
        let entry = &EnsemblHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry, &snps).await;
        let ranges_ucsc = run_ranges(entry, &ranges).await;

        cache::store(snps_ucsc, ranges_ucsc, "ensembl", key);
    }

    Ok(())
}
#[ignore]
#[tokio::test]
async fn check_testpoints_ucsc() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");
        let key = &UcscHG::key(from, to);
        let entry = &UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry, &snps).await;
        let ranges_ucsc = run_ranges(entry, &ranges).await;

        cache::assert(snps_ucsc, ranges_ucsc, "ucsc", key);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");
        let key = &EnsemblHG::key(from, to);
        let entry = &EnsemblHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps(entry, &snps).await;
        let ranges_ucsc = run_ranges(entry, &ranges).await;

        cache::assert(snps_ucsc, ranges_ucsc, "ensembl", key);
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
        let key = &UcscHG::key(from, to);
        let entry = &UcscHG::global_cache(from, to);

        let liftover = &Liftover::read_file(entry).unwrap();

        let (snps, ranges) = super::testpoints::get(liftover);

        let snps_ucsc = run_snps_web(from, to, &snps).await;
        let ranges_ucsc = run_ranges_web(from, to, &ranges).await;

        cache::assert(snps_ucsc, ranges_ucsc, "ucsc", key);
    }

    Ok(())
}

pub async fn run_snps(
    liftover_path: impl AsRef<Path>,
    snps: &[GenomePosition],
) -> Vec<Vec<GenomePosition>> {
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
    ranges: &[GenomeRange],
) -> Vec<Vec<GenomeRange>> {
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
    snps: &[GenomePosition],
) -> Vec<Vec<GenomePosition>> {
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
    ranges: &[GenomeRange],
) -> Vec<Vec<GenomeRange>> {
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

    use biocore::location::{GenomePosition, GenomeRange};
    use utile::cache::CacheEntry;

    pub fn get(prefix: &str, key: &str) -> (Vec<Vec<GenomePosition>>, Vec<Vec<GenomeRange>>) {
        (
            snp_testpoints_ucsc_target(prefix, key)
                .get_as_json_lines()
                .unwrap(),
            range_testpoints_ucsc_target(prefix, key)
                .get_as_json_lines()
                .unwrap(),
        )
    }
    pub fn store(
        snps_internal: Vec<Vec<GenomePosition>>,
        ranges_internal: Vec<Vec<GenomeRange>>,
        prefix: &str,
        key: &str,
    ) {
        snp_testpoints_ucsc_target(prefix, key)
            .set_json_lines(snps_internal)
            .unwrap();
        range_testpoints_ucsc_target(prefix, key)
            .set_json_lines(ranges_internal)
            .unwrap();
    }
    pub fn assert(
        snps_internal: Vec<Vec<GenomePosition>>,
        ranges_internal: Vec<Vec<GenomeRange>>,
        prefix: &str,
        key: &str,
    ) {
        let (found_snps, found_ranges) = get(prefix, key);
        assert!(snps_internal == found_snps);
        assert!(ranges_internal == found_ranges);
    }

    fn snp_testpoints_ucsc_target(prefix: &str, key: &str) -> CacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("snp_testpoints_uscs.jsonl"))
    }
    fn range_testpoints_ucsc_target(prefix: &str, key: &str) -> CacheEntry {
        super::super::cache(prefix).entry(PathBuf::from(key).join("range_testpoints_uscs.jsonl"))
    }
}
