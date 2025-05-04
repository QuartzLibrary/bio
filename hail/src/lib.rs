#![feature(ascii_char)]

pub mod contig;
pub mod resource;

use resource::HailCommonResource;
use utile::resource::{RawResource, RawResourceExt};

pub async fn load_grch38_reference_genome()
-> std::io::Result<biocore::fasta::IndexedFastaReader<std::io::BufReader<std::fs::File>>> {
    let resource = HailCommonResource::grch38_reference_genome()
        .log_progress()
        .decompressed()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;
    let index_resource = HailCommonResource::grch38_reference_genome_index()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;

    biocore::fasta::IndexedFastaReader::new(
        resource.buffered().read()?,
        index_resource.decompressed().buffered().read()?,
    )
}

pub async fn load_grch37_reference_genome()
-> std::io::Result<biocore::fasta::IndexedFastaReader<std::io::BufReader<std::fs::File>>> {
    let resource = HailCommonResource::old_grch37_reference_genome()
        .log_progress()
        .with_global_fs_cache()
        .decompressed() // Decompress *before* caching, so we have a file to index into.
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;
    let index_resource = HailCommonResource::old_grch37_reference_genome_index()
        .log_progress()
        .with_global_fs_cache()
        .ensure_cached_async()
        .await?;

    biocore::fasta::IndexedFastaReader::new(
        resource.buffered().read()?,
        index_resource.decompressed().buffered().read()?,
    )
}
