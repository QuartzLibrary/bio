#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("reqwest", log::LevelFilter::Info)
        .init();

    log::info!("Starting");

    gwas_catalog::GwasCatalogStudy::get_latest()
        .await
        .unwrap()
        .for_each(|v| drop(v.unwrap()));

    gwas_catalog::GwasCatalogAncestry::get_latest()
        .await
        .unwrap()
        .for_each(|v| drop(v.unwrap()));

    gwas_catalog::GwasCatalogAssociation::get_latest()
        .await
        .unwrap()
        .for_each(|v| drop(v.unwrap()));
}
