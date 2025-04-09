pub trait Contig {
    fn name(&self) -> &str;
    fn size(&self) -> u64;
}
