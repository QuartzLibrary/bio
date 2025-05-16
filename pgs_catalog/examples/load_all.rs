use std::collections::HashSet;

use ids::pgs::PgsId;
use pgs_catalog::{metadata::Metadata, Allele, GenomeBuild, HarmonizedStudy, Study};

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("reqwest", log::LevelFilter::Info)
        .init();

    log::info!("Starting");

    Metadata::load_all().await.unwrap();

    let mut effect_alleles = HashSet::new();

    for id in PgsId::iter_test() {
        log::info!("{id}");

        Metadata::load(id).await.unwrap();

        Study::load_associations_default(id)
            .await
            .unwrap()
            .for_each(|v| drop(v.unwrap()));

        HarmonizedStudy::load_associations_default(id, GenomeBuild::GRCh37)
            .await
            .unwrap()
            .for_each(|v| drop(v.unwrap()));

        HarmonizedStudy::load_associations_default(id, GenomeBuild::GRCh38)
            .await
            .unwrap()
            .for_each(|v| {
                let v = v.unwrap();
                effect_alleles.insert(v.effect_allele.clone());
                // drop(v.unwrap())
            });
    }

    effect_alleles.retain(|v| matches!(v, Allele::Insertion | Allele::Other(_)));

    println!("{effect_alleles:?}");
}
