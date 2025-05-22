pub trait Contig: AsRef<str> {
    fn size(&self) -> u64;
}
