use genomes1000::{load_grch38_reference_genome, GRCh38Contig};
use noodles::fasta::Reader;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("reqwest", log::LevelFilter::Info)
        .filter_module("hyper_util", log::LevelFilter::Info)
        .init();

    log::info!("Starting");

    let reference_genome = load_grch38_reference_genome().await.unwrap();

    for record in Reader::new(reference_genome.into_inner()).records() {
        let _record = record.unwrap();
    }

    let tasks = GRCh38Contig::CHROMOSOMES
        .into_iter()
        .map(|c| async move {
            genomes1000::load_contig(c)
                .await
                .unwrap()
                .enumerate()
                .for_each(|(i, v)| match v {
                    Ok(v) => drop(v),
                    Err(e) => {
                        log::error!("{c}@{i}:\n{e}")
                    }
                });
        })
        .map(tokio::spawn);

    futures::future::join_all(tasks).await;
}
