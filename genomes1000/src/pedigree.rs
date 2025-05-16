use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct Pedigree {
    #[serde(rename = "SampleID")]
    pub id: String,
    #[serde(rename = "FamilyID")]
    pub family_id: String,
    #[serde(rename = "FatherID")]
    pub father_id: String,
    #[serde(rename = "MotherID")]
    pub mother_id: String,
    #[serde(rename = "Sex")]
    pub sex: Sex,
    #[serde(rename = "Population")]
    pub population: String,
    #[serde(rename = "Superpopulation")]
    pub superpopulation: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Sex {
    #[serde(rename = "1")]
    Male,
    #[serde(rename = "2")]
    Female,
}
