use std::sync::Arc;

pub trait Contig: AsRef<str> + PartialEq + Eq {
    fn size(&self) -> u64;
}
impl<C> Contig for &C
where
    C: Contig + ?Sized,
{
    fn size(&self) -> u64 {
        <C as Contig>::size(self)
    }
}

/// A generic reference-counted contig that keeps track of its size.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcContig {
    name: Arc<str>,
    size: u64,
}
impl AsRef<str> for ArcContig {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}
impl Contig for ArcContig {
    fn size(&self) -> u64 {
        self.size
    }
}
impl ArcContig {
    pub fn new(name: Arc<str>, size: u64) -> Self {
        Self { name, size }
    }
}

/// Useful as an interface any contig can be cast to without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContigRef<'a> {
    name: &'a str,
    size: u64,
}
impl<'a> AsRef<str> for ContigRef<'a> {
    fn as_ref(&self) -> &str {
        self.name
    }
}
impl<'a> Contig for ContigRef<'a> {
    fn size(&self) -> u64 {
        self.size
    }
}
impl<'a> ContigRef<'a> {
    pub fn new(name: &'a str, size: u64) -> Self {
        Self { name, size }
    }
}
impl<'a, C: Contig> From<&'a C> for ContigRef<'a> {
    fn from(c: &'a C) -> Self {
        Self::new(c.as_ref(), c.size())
    }
}
