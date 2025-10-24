use biocore::dna::{DnaSequence, IupacDnaSequence};
use serde::{Deserialize, Serialize};
use utile::num::TryU64;

// https://pmc.ncbi.nlm.nih.gov/articles/PMC6907074/
// https://pmc.ncbi.nlm.nih.gov/articles/instance/6907074/bin/NIHMS1541141-supplement-Sup_info.pdf
const DEFAULT_SCAFFOLD: &str =
    "GTTTTAGAGCTAGAAATAGCAAGTTAAAATAAGGCTAGTCCGTTATCAACTTGAAAAAGTGGCACCGAGTCGGTGC";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Editor {
    pub pam_pattern: IupacDnaSequence,
    pub nick_distance: u64,
    pub spacer_size: u64,
    pub scaffold: DnaSequence,
}
impl Editor {
    pub fn sp_cas9() -> Self {
        Self {
            pam_pattern: "NGG".parse().unwrap(),
            nick_distance: 3,
            spacer_size: 20,
            scaffold: DEFAULT_SCAFFOLD.parse().unwrap(),
        }
    }
    pub fn pam_size(&self) -> u64 {
        self.pam_pattern.len().u64_unwrap()
    }
}
