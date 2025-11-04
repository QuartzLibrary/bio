# bio

Rust crates for working with genomic data and building bioinformatics tools.

## Crates

- `biocore`: Core types and parsers for biological data: DNA/RNA sequences, amino acids, genomic locations, VCF/BCF files, FASTA readers, and mutation representations. Built on `noodles` for format support. Note that genomic coordinates are 0-indexed internally (because it makes writing software [easier](https://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD831.html)).

- `liftover`: Genome coordinate conversion between assemblies (GRCh37 ↔ GRCh38, etc.). Parses UCSC chain files and provides both naive and indexed lookups. Handles strand orientation.

- `puv`: Rust-Python interop using [PEP 723](https://peps.python.org/pep-0723/) inline script metadata. Executes typed Python functions from Rust with automatic JSON serialization and `uv`-managed dependencies.

- `primeedit`: Prime editing guide RNA design. Generates pegRNA designs from sequence edits, validates PAM sites, computes RTT templates, and checks for seed/PAM disruption. Includes silent mutation insertion for MMR evasion.

- `pgs_catalog`: Client for the [PGS Catalog](https://www.pgscatalog.org/) (Polygenic Score Catalog). Loads scoring files and harmonized variants across genome builds. Handles the catalog's complex metadata and provides simplified representations for downstream analysis.

- `gwas_catalog`: Client for the [GWAS Catalog](https://www.ebi.ac.uk/gwas/). Fetches and deserializes association data, study metadata, and ancestry information.

- `pan_ukbb`: Loader for [Pan-UK Biobank](https://pan.ukbb.broadinstitute.org/) summary statistics.

- `genomes1000`: Parser and loader for 1000 Genomes Project high-coverage VCF data. Handles multi-allelic splitting, normalization, and querying with tabix indexes. Includes pedigree information.
Note: I handwrote the parser for fun, but you probably wants something that skips most of the metadata.

- `ids`: Newtypes for identifiers: `RsId`, `PgsId`, `PubmedId`. Provides parsing and validation.

- `ensembl` / `hail`: Resource helpers for Ensembl and Hail reference genomes with embedded contig metadata for GRCh37/GRCh38.

- `utile`: Useful utilities.

## Requirements

- Requires nightly (see `rust-toolchain.toml`).

