use utile::resource::RawResourceExt;

use genomes1000::{load_grch38_reference_genome, resource::Genomes1000Resource, GRCh38Contig};

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("reqwest", log::LevelFilter::Info)
        .filter_module("hyper_util", log::LevelFilter::Info)
        .init();

    log::info!("Starting");

    let fasta = Genomes1000Resource::grch38_reference_genome()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await
        .unwrap()
        .buffered();
    let index = Genomes1000Resource::grch38_reference_genome_index()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await
        .unwrap();

    let reference_genome = load_grch38_reference_genome(fasta, index).await.unwrap();

    for record in reference_genome.into_records() {
        let _record = record.unwrap();
    }

    let tasks = GRCh38Contig::CHROMOSOMES
        .into_iter()
        .map(|c| async move {
            let (_sample_names, records) = genomes1000::load_contig(c).await.unwrap();
            records.enumerate().for_each(|(i, v)| match v {
                Ok(v) => drop(v),
                Err(e) => {
                    log::error!("{c}@{i}:\n{e}")
                }
            });
        })
        .map(tokio::spawn);

    futures::future::join_all(tasks).await;
}
