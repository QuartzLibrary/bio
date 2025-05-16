use std::{
    io::{self, BufRead},
    str,
};

use biocore::dna::DnaBase;
use utile::io::FromUtf8Bytes;

use crate::GRCh38Contig;

use super::Record;

pub(super) fn parse<S>(
    reader: impl BufRead,
    read_sample: fn(&[u8], &[u8]) -> io::Result<S>,
) -> io::Result<(Vec<String>, Lines<S, impl BufRead>)> {
    let mut reader = comments::skip(reader)?;
    let sample_names = read_header(&mut reader)?;

    Ok((
        sample_names.clone(),
        Lines {
            buf: vec![],
            inner: reader,
            sample_names,
            read_sample,
        },
    ))
}

pub(super) fn read_header(reader: &mut impl BufRead) -> Result<Vec<String>, io::Error> {
    const EXPECTED_SAMPLE_COUNT: usize = 2500;
    const REFERENCE: [u8; 46] = *b"#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t";
    let mut buf = [0; REFERENCE.len()];
    reader.read_exact(&mut buf)?;
    assert_eq!(REFERENCE, buf, "{:?}", str::from_utf8(&buf));

    let mut sample_names = Vec::with_capacity(EXPECTED_SAMPLE_COUNT);
    let mut line = Vec::with_capacity(EXPECTED_SAMPLE_COUNT * 8);
    reader.read_until(b'\n', &mut line)?;
    assert_eq!(Some(&b'\n'), line.last());
    line.pop();
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    let line = String::from_utf8(line).map_err(|_| {
        log::warn!("[Data][1000 Genomes] File header is not valid UTF-8.");
        io::ErrorKind::InvalidData
    })?;
    sample_names.extend(line.split('\t').map(str::to_owned));
    Ok(sample_names)
}

#[derive(Debug)]
pub(super) struct Lines<S, B> {
    buf: Vec<u8>,
    inner: B,
    sample_names: Vec<String>,
    read_sample: fn(&[u8], &[u8]) -> io::Result<S>,
}
impl<S, B> Iterator for Lines<S, B>
where
    B: BufRead,
{
    type Item = Result<Record<S>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_record(
            &mut self.buf,
            &self.sample_names,
            &mut self.inner,
            self.read_sample,
        ) {
            Ok(Some(r)) => Some(Ok(r)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub(super) fn read_record<S>(
    buf: &mut Vec<u8>,
    sample_names: &[String],
    reader: &mut impl BufRead,
    read_sample: fn(&[u8], &[u8]) -> Result<S, io::Error>,
) -> Result<Option<Record<S>>, io::Error> {
    fn take_string(buf: &mut Vec<u8>, reader: &mut impl BufRead) -> Result<String, io::Error> {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        let buf = &buf[..buf.len() - 1];
        Ok(String::from_utf8(buf.to_vec()).unwrap())
    }

    let contig: GRCh38Contig = {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        if buf.is_empty() {
            return Ok(None);
        }
        let buf = &buf[..buf.len() - 1];
        GRCh38Contig::from_bytes(buf).unwrap()
    };
    let position: u64 = {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        let buf = &buf[..buf.len() - 1];
        str::from_utf8(buf).unwrap().parse().unwrap()
    };
    let id = take_string(buf, reader)?;
    let reference_allele = {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        let buf = &buf[..buf.len() - 1];
        {
            let mut genotype = Vec::with_capacity(buf.len());

            for base in buf {
                let base = match base {
                    b'A' => Some(DnaBase::A),
                    b'C' => Some(DnaBase::C),
                    b'G' => Some(DnaBase::G),
                    b'T' => Some(DnaBase::T),
                    b'N' => None,
                    _ => todo!(),
                };
                genotype.push(base);
            }

            genotype
        }
    };
    let alternate_alleles: Vec<crate::AltGenotype> = {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        let buf = &buf[..buf.len() - 1];
        utile::io::parse::string_sequence::buf(buf, b',').unwrap()
    };
    let quality: Option<f64> = {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        let buf = &buf[..buf.len() - 1];
        if buf == b"." {
            None
        } else {
            Some(f64::from_bytes(buf).unwrap())
        }
    };
    let filter = take_string(buf, reader)?;
    let info = take_string(buf, reader)?;
    let format = take_string(buf, reader)?;

    let mut samples = Vec::with_capacity(sample_names.len());

    for _ in 0..sample_names.len() - 1 {
        buf.clear();
        reader.read_until(b'\t', buf)?;
        let buf = &buf[..buf.len() - 1];
        samples.push(read_sample(format.as_bytes(), buf)?);
    }

    {
        buf.clear();
        reader.read_until(b'\n', buf)?;
        let mut buf = &**buf;
        if buf.last() == Some(&b'\n') {
            buf = &buf[..buf.len() - 1];
        }
        if buf.last() == Some(&b'\r') {
            buf = &buf[..buf.len() - 1];
        }
        samples.push(read_sample(format.as_bytes(), buf)?);
    }

    Ok(Some(Record {
        contig,
        position,
        id,
        reference_allele,
        alternate_alleles,
        quality,
        filter,
        info,
        format,
        samples,
    }))
}

mod ysample {
    use std::io;

    use utile::io::FromUtf8Bytes;

    use crate::ExtendedSample;

    impl<GT> ExtendedSample<GT>
    where
        GT: FromUtf8Bytes,
        io::Error: From<<GT as FromUtf8Bytes>::Err>,
    {
        /// GT:CN:CNL:CNP:CNQ:GP:GQ:PL
        /// 0:1:-296.36,0,-16.6:-300.46,0,-19.7:99:0,-19.7:99:0,166
        /// 0:1:-1000,0,-39.44:-1000,0,-42.54:99:0,-42.54:99:0,394
        #[allow(non_snake_case)]
        pub fn from_bytes(raw_keys: &[u8], buf: &[u8]) -> io::Result<Self> {
            let keys = raw_keys.split(|v| v == &b':');
            let mut values = buf.split(|v| v == &b':');

            let mut AB: Option<f64> = None;
            let mut AD: Option<Vec<u64>> = None;
            let mut DP: Option<u64> = None;
            let mut GQ: Option<u64> = None;
            let mut GT: Option<Option<GT>> = None;
            let mut MIN_DP: Option<u64> = None;
            let mut MQ0: Option<u64> = None;
            let mut PGT: Option<String> = None;
            let mut PID: Option<String> = None;
            let mut PL: Option<Vec<u64>> = None;
            let mut RGQ: Option<u64> = None;
            let mut SB: Option<Vec<u64>> = None;

            for key in keys {
                let Some(value) = values.next() else {
                    // Unfortunately some of the entries in the `others` vcf file are malformed.
                    // Hopefully the missing ones are at the tail at least.
                    break;
                    #[allow(unreachable_code)]
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Missing value for key. Format: {:?}. Value: {:?}.",
                            std::str::from_utf8(raw_keys),
                            std::str::from_utf8(buf)
                        ),
                    ));
                };
                match key {
                    b"GT" if GT.is_none() && value == [b'.'] => GT = Some(None),
                    _ if value == [b'.'] => continue,

                    b"GT" if GT.is_none() => GT = Some(Some(GT::from_bytes(value)?)),

                    b"AB" if AB.is_none() => AB = Some(f64::from_bytes(value)?),
                    b"AD" if AD.is_none() => {
                        AD = Some(utile::io::parse::string_sequence::buf(value, b',')?)
                    }
                    b"DP" if DP.is_none() => DP = Some(u64::from_bytes(value)?),
                    b"GQ" if GQ.is_none() => GQ = Some(u64::from_bytes(value)?),
                    b"MIN_DP" if MIN_DP.is_none() => MIN_DP = Some(u64::from_bytes(value)?),
                    b"MQ0" if MQ0.is_none() => MQ0 = Some(u64::from_bytes(value)?),
                    b"PGT" if PGT.is_none() => PGT = Some(String::from_bytes(value)?),
                    b"PID" if PID.is_none() => PID = Some(String::from_bytes(value)?),
                    b"PL" if PL.is_none() => {
                        PL = Some(utile::io::parse::string_sequence::buf(value, b',')?)
                    }
                    b"RGQ" if RGQ.is_none() => RGQ = Some(u64::from_bytes(value)?),
                    b"SB" if SB.is_none() => {
                        SB = Some(utile::io::parse::string_sequence::buf(value, b',')?)
                    }

                    _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected key")),
                }
            }

            if values.next().is_some() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Extra value"));
            }

            Ok(Self {
                AB,
                AD: AD.unwrap(),
                DP,
                GQ,
                GT: GT.unwrap(),
                MIN_DP,
                MQ0,
                PGT,
                PID,
                PL,
                RGQ,
                SB,
            })
        }
    }
}

mod genotype {
    use std::{io, str, str::FromStr};

    use utile::io::FromUtf8Bytes;

    use crate::{DiploidGenotype, Genotype, GenotypePhasing, HaploidGenotype};

    use super::utf8_error;

    impl FromUtf8Bytes for Genotype {
        type Err = io::Error;

        fn from_bytes(buf: &[u8]) -> Result<Self, io::Error> {
            match buf {
                [b'.'] => Ok(Self::Missing),
                [b'0'..=b'9'] => Ok(Self::Haploid(HaploidGenotype {
                    value: buf[0] - b'0',
                })),
                [b'0'..=b'9', b'|', b'0'..=b'9'] => {
                    let left = buf[0] - b'0';
                    let right = buf[2] - b'0';
                    Ok(Self::Diploid(DiploidGenotype {
                        left,
                        phasing: GenotypePhasing::Phased,
                        right,
                    }))
                }
                _ => Ok(str::from_utf8(buf)
                    .map_err(|e| utf8_error("Genotype", e))?
                    .parse()?),
            }
        }
    }
    impl FromStr for Genotype {
        type Err = io::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            fn error(s: &str) -> io::Error {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Expected Genotype, but found \"{s}\"."),
                )
            }
            if s == "." || s == "./." {
                return Ok(Self::Missing);
            }
            match s.parse::<DiploidGenotype>() {
                Ok(diploid) => Ok(Self::Diploid(diploid)),
                Err(_) => match s.parse::<HaploidGenotype>() {
                    Ok(haploid) => Ok(Self::Haploid(haploid)),
                    Err(_) => Err(error(s)),
                },
            }
        }
    }

    impl FromUtf8Bytes for HaploidGenotype {
        type Err = io::Error;

        fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
            match bytes {
                [b'0'..=b'9'] => Ok(Self {
                    value: bytes[0] - b'0',
                }),
                _ => Ok(str::from_utf8(bytes)
                    .map_err(|e| utf8_error("HaploidGenotype", e))?
                    .parse()?),
            }
        }
    }
    impl FromStr for HaploidGenotype {
        type Err = io::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            fn error(s: &str) -> io::Error {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Expected HaploidGenotype, but found \"{s}\"."),
                )
            }
            Ok(Self {
                value: s.parse().map_err(|_| error(s))?,
            })
        }
    }

    impl DiploidGenotype {
        pub fn from_bytes(buf: &[u8]) -> Result<Self, io::Error> {
            if buf.len() == 3 {
                let left = buf[0] - b'0';
                assert_eq!(b'|', buf[1]);
                let right = buf[2] - b'0';
                Ok(Self {
                    left,
                    phasing: GenotypePhasing::Phased,
                    right,
                })
            } else {
                Ok(str::from_utf8(buf)
                    .map_err(|e| utf8_error("DiploidGenotype", e))?
                    .parse()?)
            }
        }
    }
    impl FromStr for DiploidGenotype {
        type Err = io::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            fn error(s: &str) -> io::Error {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Expected DiploidGenotype, but found \"{s}\"."),
                )
            }
            let mut split = s.split('|');
            match (split.next(), split.next()) {
                (None, None | Some(_)) => unreachable!(),
                (Some(_), None) => {
                    let mut split = s.split('/');
                    match (split.next(), split.next()) {
                        (None, None | Some(_)) => unreachable!(),
                        (Some(_), None) => Err(error(s)),
                        (Some(left), Some(right)) => {
                            if split.next().is_some() {
                                Err(error(s))?;
                            }
                            Ok(Self {
                                left: left.parse().map_err(|_| error(s))?,
                                phasing: GenotypePhasing::Unphased,
                                right: right.parse().map_err(|_| error(s))?,
                            })
                        }
                    }
                }
                (Some(left), Some(right)) => {
                    if split.next().is_some() {
                        Err(error(s))?;
                    }
                    Ok(Self {
                        left: left.parse().map_err(|_| error(s))?,
                        phasing: GenotypePhasing::Phased,
                        right: right.parse().map_err(|_| error(s))?,
                    })
                }
            }
        }
    }
}

pub mod comments {
    use std::io::{self, Read};

    #[allow(dead_code)]
    pub fn read(reader: &mut impl io::BufRead) -> io::Result<String> {
        let mut vec = vec![];

        while let [b'#', b'#', ..] = reader.fill_buf()? {
            reader.read_until(b'\n', &mut vec)?;
        }

        String::from_utf8(vec).map_err(utile::io::invalid_data)
    }
    pub fn skip<R: io::BufRead>(mut reader: R) -> io::Result<io::Chain<io::Cursor<[u8; 2]>, R>> {
        let mut buf = [0, 0];

        while let [b'#', b'#'] = {
            reader.read_exact(&mut buf)?;
            buf
        } {
            reader.skip_until(b'\n')?;
        }

        Ok(io::Cursor::new(buf).chain(reader))
    }

    #[test]
    fn test_skip_comments() -> io::Result<()> {
        use io::{Cursor, Read};

        let mut reader = Cursor::new(
            "##Comment 1\n##Comment 2\n##Comment 3\n#Header\nMore content\n##Ignored comment",
        );
        skip(&mut reader)?;

        let mut v = String::new();
        reader.read_to_string(&mut v)?;
        assert_eq!(v, "#Header\nMore content\n##Ignored comment");

        Ok(())
    }
}

fn utf8_error(type_: &'static str, e: str::Utf8Error) -> io::Error {
    println!("{e:?}");
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Expected {type_}, but found invalid UTF-8: {e:?}."),
    )
}
