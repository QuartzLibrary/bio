use biocore::{
    dna::DnaSequence,
    location::{GenomePosition, SequenceOrientation},
};

use crate::{GRCh38Contig, Genotype};

#[derive(Debug, Clone)]
pub struct SimplifiedRecord {
    pub contig: GRCh38Contig,
    /// 1-based! 0 and n+1 means telomere (where n is length of contig).
    pub position: u64,
    pub reference_allele: DnaSequence,
    pub alternate_allele: DnaSequence,
    // pub id: String,
    pub quality: Option<f64>,
    pub filter: String,
    pub samples: Vec<Genotype>,
}

impl SimplifiedRecord {
    pub fn at(&self) -> GenomePosition<GRCh38Contig> {
        GenomePosition {
            name: self.contig,
            orientation: SequenceOrientation::Forward,
            at: self.position - 1,
        }
    }
}
