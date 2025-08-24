use core::fmt;
use std::{
    collections::BTreeMap,
    io::{self, BufRead, Seek},
    str::FromStr,
};

use noodles::fasta::{
    io::{reader::Records, Reader},
    record::Definition,
};

use crate::{
    genome::InMemoryGenome,
    location::{ContigPosition, GenomeRange},
    sequence::{AsciiChar, Sequence},
};

pub struct IndexedFastaReader<R> {
    reader: Reader<R>,
    index: noodles::fasta::fai::Index,
}

impl<R: BufRead> IndexedFastaReader<R> {
    pub fn new(reader: R, index: impl BufRead) -> io::Result<Self> {
        Ok(Self {
            reader: Reader::new(reader),
            index: noodles::fasta::fai::io::Reader::new(index).read_index()?,
        })
    }

    pub fn query_position<B: AsciiChar>(&mut self, loc: &ContigPosition) -> io::Result<Sequence<B>>
    where
        R: Seek,
    {
        self.query(&loc.clone().into())
    }

    pub fn query<B, C>(&mut self, at: &GenomeRange<C>) -> io::Result<Sequence<B>>
    where
        R: Seek,
        B: AsciiChar,
        C: AsRef<str> + Clone,
    {
        let record = self
            .reader
            .query(&self.index, &at.clone().try_into().unwrap())?;
        let sequence = record.sequence();
        // TODO: avoid clone
        Sequence::<B>::try_from(sequence.clone()).map_err(Into::into)
    }

    pub fn records(&mut self) -> Records<'_, R> {
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
    inner: Reader<R>,
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

impl<C, B> InMemoryGenome<C, B> {
    pub fn from_fasta(reader: impl BufRead) -> Result<Self, FastaGenomeError<C, B>>
    where
        C: FromStr + Ord,
        <C as FromStr>::Err: fmt::Debug,
        B: AsciiChar,
        <B as AsciiChar>::DecodeError: fmt::Debug,
    {
        let mut reader = Reader::new(reader);

        let mut definition = String::new();

        let mut contigs = BTreeMap::new();

        while reader.read_definition(&mut definition)? > 0 {
            let def: Definition = definition.parse()?;
            let contig = C::from_str(def.name().try_into().unwrap())
                .map_err(FastaGenomeError::InvalidContigName)?;

            let mut sequence = Vec::new();
            reader.read_sequence(&mut sequence)?;
            let sequence = B::decode(sequence).map_err(Into::into)?;

            contigs.insert(contig, sequence);

            definition.clear();
        }

        Ok(Self { contigs })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FastaGenomeError<C, B>
where
    C: FromStr,
    <C as FromStr>::Err: fmt::Debug,
    B: AsciiChar,
    <B as AsciiChar>::DecodeError: fmt::Debug,
{
    #[error(transparent)]
    InvalidDefinition(noodles::fasta::record::definition::ParseError),
    #[error("Invalid contig name: {0}")]
    InvalidContigName(<C as FromStr>::Err),
    #[error("Invalid sequence: {0}")]
    InvalidSequence(<B as AsciiChar>::DecodeError),
    #[error(transparent)]
    Io(io::Error),
}
impl<C, B> From<noodles::fasta::record::definition::ParseError> for FastaGenomeError<C, B>
where
    C: FromStr,
    <C as FromStr>::Err: fmt::Debug,
    B: AsciiChar,
    <B as AsciiChar>::DecodeError: fmt::Debug,
{
    fn from(e: noodles::fasta::record::definition::ParseError) -> Self {
        Self::InvalidDefinition(e)
    }
}

impl<C, B> From<io::Error> for FastaGenomeError<C, B>
where
    C: FromStr,
    <C as FromStr>::Err: fmt::Debug,
    B: AsciiChar,
    <B as AsciiChar>::DecodeError: fmt::Debug,
{
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
