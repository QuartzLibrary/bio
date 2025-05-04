use std::io::{self, BufRead, Seek};

use crate::{
    location::{GenomePosition, GenomeRange},
    sequence::{AsciiChar, Sequence},
};

pub struct IndexedFastaReader<R> {
    reader: noodles::fasta::Reader<R>,
    index: noodles::fasta::fai::Index,
}

impl<R: BufRead> IndexedFastaReader<R> {
    pub fn new(reader: R, index: impl BufRead) -> io::Result<Self> {
        Ok(Self {
            reader: noodles::fasta::Reader::new(reader),
            index: noodles::fasta::fai::Reader::new(index).read_index()?,
        })
    }

    pub fn query_position<C: AsciiChar>(&mut self, loc: &GenomePosition) -> io::Result<Sequence<C>>
    where
        R: Seek,
    {
        self.query(&loc.clone().into())
    }

    pub fn query<C: AsciiChar>(&mut self, at: &GenomeRange) -> io::Result<Sequence<C>>
    where
        R: Seek,
    {
        let record = self
            .reader
            .query(&self.index, &at.clone().try_into().unwrap())?;
        let sequence = record.sequence();
        // TODO: avoid clone
        Sequence::<C>::try_from(sequence.clone()).map_err(Into::into)
    }

    pub fn records(&mut self) -> noodles::fasta::reader::Records<'_, R> {
        self.reader.records()
    }

    pub fn into_records(self) -> IntoRecords<R> {
        IntoRecords {
            inner: self.reader,
            line_buf: String::new(),
        }
    }
}

pub struct IntoRecords<R> {
    inner: noodles::fasta::Reader<R>,
    line_buf: String,
}
impl<R> Iterator for IntoRecords<R>
where
    R: BufRead,
{
    type Item = io::Result<noodles::fasta::Record>;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_buf.clear();

        match self.inner.read_definition(&mut self.line_buf) {
            Ok(0) => return None,
            Ok(_) => {}
            Err(e) => return Some(Err(e)),
        }

        let definition = match self.line_buf.parse() {
            Ok(d) => d,
            Err(e) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
        };

        let mut sequence_buf = Vec::new();

        match self.inner.read_sequence(&mut sequence_buf) {
            Ok(_) => {
                let record = noodles::fasta::Record::new(
                    definition,
                    noodles::fasta::record::Sequence::from(sequence_buf),
                );
                Some(Ok(record))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
