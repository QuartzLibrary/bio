use genomes1000::B38Contig;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("reqwest", log::LevelFilter::Info)
        .filter_module("hyper_util", log::LevelFilter::Info)
        .init();

    log::info!("Starting");

    let tasks = B38Contig::iter()
        .into_iter()
        .map(|c| async move {
            genomes1000::load_contig(c.clone())
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
