use std::io::{BufRead, BufReader, Seek};

use url::Url;
use utile::cache::{Cache, CacheEntry, UrlEntry};

const REF_URL: &str = "https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/technical/reference/GRCh38_reference_genome/GRCh38_full_analysis_set_plus_decoy_hla.fa";
const REF_INDEX_URL: &str = "https://ftp.1000genomes.ebi.ac.uk/vol1/ftp/technical/reference/GRCh38_reference_genome/GRCh38_full_analysis_set_plus_decoy_hla.fa.fai";

pub async fn get_reference_genome(
) -> std::io::Result<noodles::fasta::IndexedReader<impl BufRead + Seek>> {
    let fs_entry = download_and_cache().await?;
    let index_fs_entry = download_and_cache_index().await?;

    Ok(noodles::fasta::IndexedReader::new(
        BufReader::new(fs_entry.get()?),
        noodles::fasta::fai::read(index_fs_entry)?,
    ))
}

async fn download_and_cache() -> Result<CacheEntry, std::io::Error> {
    let url: Url = REF_URL.parse().unwrap();
    let file_name = url.path_segments().unwrap().last().unwrap();

    let fs_entry = Cache::global("1000genomes").entry(file_name);

    UrlEntry::new(url)
        .unwrap()
        .get_and_cache_async("[Data][1000 Genomes]", fs_entry.clone())
        .await?;

    Ok(fs_entry)
}
async fn download_and_cache_index() -> Result<CacheEntry, std::io::Error> {
    let url: Url = REF_INDEX_URL.parse().unwrap();
    let file_name = url.path_segments().unwrap().last().unwrap();

    let fs_entry = Cache::global("1000genomes").entry(file_name);

    UrlEntry::new(url)
        .unwrap()
        .get_and_cache_async("[Data][1000 Genomes]", fs_entry.clone())
        .await?;

    Ok(fs_entry)
}
