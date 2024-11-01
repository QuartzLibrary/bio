use liftover::{
    sources::{EnsemblHG, UcscHG},
    Liftover,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    for (from, to) in UcscHG::valid_pairs() {
        println!("{from} {to}");

        // We use the global cache to avoid re-distributing the file in the repo.
        let liftover = Liftover::load_human_ucsc(from, to).await.unwrap();

        drop(liftover);
    }

    for (from, to) in EnsemblHG::valid_pairs() {
        println!("{from} {to}");

        // We use the global cache to avoid re-distributing the file in the repo.
        let liftover = Liftover::load_human_ensembl(from, to).await.unwrap();

        drop(liftover);
    }

    Ok(())
}
