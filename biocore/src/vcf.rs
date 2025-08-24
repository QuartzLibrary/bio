use std::{
    io::{self, BufRead, Read, Seek},
    ops::Range,
};

use noodles::csi::BinningIndex;
use utile::range::RangeExt;

use crate::location::ContigRange;

pub struct IndexedVcfReader<R> {
    header: noodles::vcf::Header,
    reader: noodles::vcf::io::Reader<noodles::bgzf::io::Reader<R>>,
    index: noodles::tabix::Index,
}
impl<R: Read> IndexedVcfReader<R> {
    pub fn new(reader: R, index: impl Read) -> io::Result<Self> {
        let mut reader = noodles::vcf::io::Reader::new(noodles::bgzf::io::Reader::new(reader));
        let header = reader.read_header()?;
        let index = noodles::tabix::io::Reader::new(index).read_index()?;
        Ok(Self {
            header,
            reader,
            index,
        })
    }

    pub fn header(&self) -> &noodles::vcf::Header {
        &self.header
    }

    pub fn query<C>(
        &mut self,
        at: &ContigRange<C>,
    ) -> io::Result<Query<noodles::bgzf::io::Reader<R>>>
    where
        R: Seek,
        C: AsRef<str>,
    {
        let reference_sequence_name = at.contig.as_ref().as_bytes().to_vec();

        let reference_sequence_id = resolve_region(&self.index, at.contig.as_ref())?;
        let chunks = self
            .index
            .query(reference_sequence_id, at.try_into().unwrap())?;

        Ok(Query::new(
            self.reader.get_mut(),
            chunks,
            reference_sequence_name,
            at.at.clone(),
        ))
    }

    pub fn query_raw<C>(
        &mut self,
        at: &ContigRange<C>,
    ) -> io::Result<QueryRaw<noodles::bgzf::io::Reader<R>>>
    where
        R: Seek,
        C: AsRef<str>,
    {
        let reference_sequence_name = at.contig.as_ref().as_bytes().to_vec();

        let reference_sequence_id = resolve_region(&self.index, at.contig.as_ref())?;
        let chunks = self
            .index
            .query(reference_sequence_id, at.try_into().unwrap())?;

        Ok(QueryRaw::new(
            self.reader.get_mut(),
            chunks,
            reference_sequence_name,
            at.at.clone(),
        ))
    }
}

pub(crate) fn resolve_region(index: &impl BinningIndex, name: &str) -> io::Result<usize> {
    let header = index
        .header()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing tabix header"))?;

    let i = header
        .reference_sequence_names()
        .get_index_of(name.as_bytes())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "region reference sequence does not exist in reference sequences: {name:?}"
                ),
            )
        })?;

    Ok(i)
}

pub struct Query<'r, R> {
    reader: noodles::vcf::io::Reader<noodles::csi::io::Query<'r, R>>,
    record: noodles::vcf::Record,

    reference_sequence_name: Vec<u8>,
    range: Range<u64>,
}

impl<'r, R> Query<'r, R>
where
    R: noodles::bgzf::io::BufRead + noodles::bgzf::io::Seek,
{
    pub(super) fn new(
        reader: &'r mut R,
        chunks: Vec<noodles::csi::binning_index::index::reference_sequence::bin::Chunk>,
        reference_sequence_name: Vec<u8>,
        range: Range<u64>,
    ) -> Self {
        Self {
            reader: noodles::vcf::io::Reader::new(noodles::csi::io::Query::new(reader, chunks)),
            reference_sequence_name,
            range,
            record: noodles::vcf::Record::default(),
        }
    }
}
impl<R> Iterator for Query<'_, R>
where
    R: noodles::bgzf::io::BufRead + noodles::bgzf::io::Seek,
{
    type Item = io::Result<noodles::vcf::Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_record(&mut self.record) {
                Ok(0) => return None,
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            };

            match intersects(&self.record, &self.reference_sequence_name, &self.range) {
                Ok(false) => continue,
                Ok(true) => return Some(Ok(self.record.clone())),
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

pub struct QueryRaw<'r, R> {
    reader: noodles::vcf::io::Reader<noodles::csi::io::Query<'r, R>>,
    record: Vec<u8>,

    reference_sequence_name: Vec<u8>,
    range: Range<u64>,
}

impl<'r, R> QueryRaw<'r, R>
where
    R: noodles::bgzf::io::BufRead + noodles::bgzf::io::Seek,
{
    pub(super) fn new(
        reader: &'r mut R,
        chunks: Vec<noodles::csi::binning_index::index::reference_sequence::bin::Chunk>,
        reference_sequence_name: Vec<u8>,
        range: Range<u64>,
    ) -> Self {
        Self {
            reader: noodles::vcf::io::Reader::new(noodles::csi::io::Query::new(reader, chunks)),
            reference_sequence_name,
            range,
            record: vec![],
        }
    }
}
impl<R> Iterator for QueryRaw<'_, R>
where
    R: noodles::bgzf::io::BufRead + noodles::bgzf::io::Seek,
{
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.record.clear();

            let read = self.reader.get_mut().read_until(b'\n', &mut self.record);

            match read {
                Ok(0) => return None,
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            };

            // TODO: avoid parsing here to intersect.
            let record = match noodles::vcf::Record::try_from(&*self.record) {
                Ok(record) => record,
                Err(e) => return Some(Err(e)),
            };

            match intersects(&record, &self.reference_sequence_name, &self.range) {
                Ok(false) => continue,
                Ok(true) => return Some(Ok(self.record.clone())),
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

fn intersects(
    record: &noodles::vcf::Record,
    reference_sequence_name: &[u8],
    range: &Range<u64>,
) -> io::Result<bool> {
    if record.reference_sequence_name().as_bytes() != reference_sequence_name {
        return Ok(false);
    }

    let Some(start) = record.variant_start().transpose()? else {
        return Ok(false);
    };
    let start = start.get() - 1;

    let record_interval = start..(start + record.reference_bases().len()); // Changed here to ignore the `END` field.
    let record_interval =
        record_interval.start.try_into().unwrap()..record_interval.end.try_into().unwrap();

    Ok(record_interval.overlaps(range))
}

impl<R: std::fmt::Debug> std::fmt::Debug for IndexedVcfReader<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexedVcfReader")
            .field("header", &self.header)
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}
