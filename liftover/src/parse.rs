use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    str,
};

use biocore::{genome::ArcContig, location::orientation::WithOrientation};
use flate2::read::MultiGzDecoder;

use utile::{
    io::read_ext::AsyncReadInto,
    resource::{RawResource, RawResourceExt},
};

use super::{
    AlignmentBlock, Chain, ChainHeader, ChainRange, GenomeRange, Liftover, SequenceOrientation,
};

impl Liftover<ArcContig, ArcContig> {
    pub fn load(resource: impl RawResource) -> anyhow::Result<Self> {
        Ok(Self::read(resource.decompressed().buffered().read()?)?)
    }
    pub async fn load_async(resource: impl RawResource) -> anyhow::Result<Self> {
        Ok(Self::read(
            &*resource
                .decompressed()
                .buffered()
                .read_async()
                .await?
                .read_into_vec()
                .await?,
        )?)
    }

    pub fn read(mut reader: impl BufRead) -> Result<Self, std::io::Error> {
        let mut chains = vec![];
        let mut buf = vec![];

        let mut contigs_from = BTreeSet::new();
        let mut contigs_to = BTreeSet::new();

        while let Some(chain) =
            read_section(&mut buf, &mut reader, &mut contigs_from, &mut contigs_to)?
        {
            chains.push(chain)
        }

        check_contigs(&contigs_from)?;
        check_contigs(&contigs_to)?;

        let contigs = contigs_from
            .into_iter()
            .map(|c| (c.as_ref().to_owned(), c))
            .collect();

        Ok(Self { chains, contigs })
    }
    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(MultiGzDecoder::new(file));

        Self::read(reader)
    }
    pub fn read_gz_compressed(data: impl Read) -> Result<Self, std::io::Error> {
        let reader = BufReader::new(MultiGzDecoder::new(data));

        Self::read(reader)
    }
}
fn read_section(
    buf: &mut Vec<u8>,
    reader: &mut impl BufRead,
    contigs_from: &mut BTreeSet<ArcContig>,
    contigs_to: &mut BTreeSet<ArcContig>,
) -> Result<Option<Chain<ArcContig, ArcContig>>, std::io::Error> {
    loop {
        let preview = reader.fill_buf()?;
        return match preview {
            [b'c', ..] => Ok(Some(read_chain(buf, reader, contigs_from, contigs_to)?)),
            [] => Ok(None),
            [b'\r' | b'\n' | b'#', ..] => {
                reader.read_until(b'\n', buf)?;
                continue;
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid chain format: {:?}", str::from_utf8(buf)),
            )),
        };
    }
}
fn read_chain(
    buf: &mut Vec<u8>,
    reader: &mut impl BufRead,
    contigs_from: &mut BTreeSet<ArcContig>,
    contigs_to: &mut BTreeSet<ArcContig>,
) -> Result<Chain<ArcContig, ArcContig>, std::io::Error> {
    {
        buf.clear();
        reader.read_until(b' ', buf)?;
        match &**buf {
            b"chain " => {}
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid chain format: {:?}", str::from_utf8(buf)),
                ));
            }
        }
    }

    let header = ChainHeader {
        score: utile::io::read::from_str(buf, reader, b' ')?,
        t: read_chain_side(buf, reader, contigs_from)?,
        q: read_chain_side(buf, reader, contigs_to)?,
        id: utile::io::read::line(buf, reader)?,
    };

    // Read alignment blocks
    let mut blocks = Vec::new();
    let mut last_block = None;

    loop {
        buf.clear();
        reader.read_until(b'\n', buf)?;

        let buf = buf.trim_ascii();

        let mut parts = {
            // Seems like UCSC uses tab as a separator here (though not in the header), but ensembl uses a space.
            let separator = if buf.contains(&b' ') { b' ' } else { b'\t' };
            buf.split(move |&v| v == separator).map(|v| v.trim_ascii())
        };

        // A number should be present
        let size: u64 = utile::io::parse::buf(
            parts
                .next()
                .expect("split always returns at least one element"),
        )?;
        let dt: i64 = match parts.next() {
            None => {
                assert_eq!(None, last_block);
                last_block = Some(size);
                break;
            }
            Some(dt) => utile::io::parse::buf(dt)?,
        };
        let dq: u64 = match parts.next() {
            None => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "todo"))?,
            Some(dq) => utile::io::parse::buf(dq)?,
        };

        if parts.next().is_some() {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "todo"))?;
        }

        blocks.push(AlignmentBlock { size, dt, dq });
    }

    let Some(last_block) = last_block else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "todo"));
    };

    let chain = Chain {
        header,
        blocks,
        last_block,
    };

    Ok(chain)
}

fn read_chain_side(
    buf: &mut Vec<u8>,
    reader: &mut impl BufRead,
    contigs: &mut BTreeSet<ArcContig>,
) -> Result<ChainRange<ArcContig>, std::io::Error> {
    let name = utile::io::read::string(buf, reader, b' ')?;
    let size = utile::io::read::from_str(buf, reader, b' ')?;
    let orientation = {
        buf.clear();
        reader.read_until(b' ', buf)?;
        let buf = &buf[..buf.len() - 1];
        parse_sequence_orientation(buf)?
    };
    let at = {
        // "The alignment start and end positions are represented as zero-based half-open intervals.
        // For example, the first 100 bases of a sequence would be represented with start position = 0
        // and end position = 100, and the next 100 bases would be represented as start position = 100
        // and end position = 200."
        let start = utile::io::read::from_str(buf, reader, b' ')?;
        let end = utile::io::read::from_str(buf, reader, b' ')?;
        start..end
    };

    let contig = contigs
        .get_or_insert(ArcContig::new(name.into(), size))
        .clone();

    Ok(WithOrientation {
        orientation,
        v: GenomeRange { name: contig, at },
    })
}

/// When the strand value is "-", position coordinates are listed in terms of the reverse-complemented sequence.
fn parse_sequence_orientation(s: &[u8]) -> Result<SequenceOrientation, std::io::Error> {
    match s {
        b"+" => Ok(SequenceOrientation::Forward),
        b"-" => Ok(SequenceOrientation::Reverse),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid sequence orientation: {:?}", str::from_utf8(s)),
        )),
    }
}

fn check_contigs(contigs: &BTreeSet<ArcContig>) -> Result<(), std::io::Error> {
    let unique_contig_name_count = contigs
        .iter()
        .map(|c| c.as_ref())
        .collect::<BTreeSet<_>>()
        .len();
    if contigs.len() != unique_contig_name_count {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Duplicate contig names in chain file: {contigs:?}"),
        ))
    } else {
        Ok(())
    }
}
