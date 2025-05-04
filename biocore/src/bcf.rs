use std::{
    io::{self, Read, Seek},
    ops::Range,
};

use noodles::{csi::BinningIndex, vcf::variant::record::ReferenceBases};

use utile::range::RangeExt;

use crate::location::GenomeRange;

pub struct IndexedBcfReader<R> {
    header: noodles::vcf::Header,
    reader: noodles::bcf::io::Reader<noodles::bgzf::Reader<R>>,
    index: noodles::csi::Index,
}
impl<R: Read> IndexedBcfReader<R> {
    pub fn new(reader: R, index: impl Read) -> io::Result<Self> {
        let mut reader = noodles::bcf::io::Reader::new(reader);
        let header = reader.read_header()?;
        let index = noodles::csi::Reader::new(index).read_index().unwrap();

        Ok(Self {
            header,
            reader,
            index,
        })
    }

    pub fn header(&self) -> &noodles::vcf::Header {
        &self.header
    }

    pub fn query(&mut self, at: &GenomeRange) -> io::Result<Query<noodles::bgzf::Reader<R>>>
    where
        R: Seek,
    {
        let reference_sequence_id = resolve_region(self.header.string_maps().contigs(), &at.name)?;
        let chunks = self
            .index
            .query(reference_sequence_id, at.try_into().unwrap())?;

        Ok(Query::new(
            &self.header,
            self.reader.get_mut(),
            chunks,
            at.name.as_bytes().to_vec(),
            at.at.clone(),
        ))
    }
}

pub(crate) fn resolve_region(
    contig_string_map: &noodles::vcf::header::string_maps::ContigStringMap,
    name: &str,
) -> io::Result<usize> {
    contig_string_map.get_index_of(name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("region does not exist in contigs: {name:?}"),
        )
    })
}

pub struct Query<'r, R> {
    header: &'r noodles::vcf::Header,
    reader: noodles::bcf::io::Reader<noodles::csi::io::Query<'r, R>>,
    record: noodles::bcf::Record,

    reference_sequence_name: Vec<u8>,
    range: Range<u64>,
}

impl<'r, R> Query<'r, R>
where
    R: noodles::bgzf::io::BufRead + noodles::bgzf::io::Seek,
{
    pub(super) fn new(
        header: &'r noodles::vcf::Header,
        reader: &'r mut R,
        chunks: Vec<noodles::csi::binning_index::index::reference_sequence::bin::Chunk>,
        reference_sequence_name: Vec<u8>,
        range: Range<u64>,
    ) -> Self {
        Self {
            header,
            reader: noodles::bcf::io::Reader::from(noodles::csi::io::Query::new(reader, chunks)),
            reference_sequence_name,
            range,
            record: noodles::bcf::Record::default(),
        }
    }
}
impl<R> Iterator for Query<'_, R>
where
    R: noodles::bgzf::io::BufRead + noodles::bgzf::io::Seek,
{
    type Item = io::Result<noodles::bcf::Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_record(&mut self.record) {
                Ok(0) => return None,
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            };

            match intersects(
                self.header,
                &self.record,
                &self.reference_sequence_name,
                &self.range,
            ) {
                Ok(false) => continue,
                Ok(true) => return Some(Ok(self.record.clone())),
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
fn intersects(
    header: &noodles::vcf::Header,
    record: &noodles::bcf::Record,
    reference_sequence_name: &[u8],
    range: &Range<u64>,
) -> io::Result<bool> {
    if record
        .reference_sequence_name(header.string_maps())?
        .as_bytes()
        != reference_sequence_name
    {
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
