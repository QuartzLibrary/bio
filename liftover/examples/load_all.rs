use liftover::{
    Liftover,
    sources::{EnsemblHG, EnsemblResource, UcscHG, UcscResource},
};
use utile::resource::RawResourceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");

        // We use the global cache to avoid re-distributing the file in the repo.
        let resource = UcscResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = Liftover::load(entry).unwrap();

        drop(liftover);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");

        // We use the global cache to avoid re-distributing the file in the repo.
        let resource = EnsemblResource::new_human_liftover(from, to);
        let entry = resource.clone().with_global_fs_cache();

        let liftover = Liftover::load(entry).unwrap();

        drop(liftover);
    }

    Ok(())
}
