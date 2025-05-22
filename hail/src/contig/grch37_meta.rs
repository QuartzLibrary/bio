#[derive(Debug, PartialEq)]
pub struct ContigMeta {
    pub ord: usize,
    pub len: u64,
}

pub static META: phf::Map<&'static str, ContigMeta> = phf::phf_map! {
    "1" => ContigMeta { ord: 0, len: 249250621 },
    "2" => ContigMeta { ord: 1, len: 243199373 },
    "3" => ContigMeta { ord: 2, len: 198022430 },
    "4" => ContigMeta { ord: 3, len: 191154276 },
    "5" => ContigMeta { ord: 4, len: 180915260 },
    "6" => ContigMeta { ord: 5, len: 171115067 },
    "7" => ContigMeta { ord: 6, len: 159138663 },
    "8" => ContigMeta { ord: 7, len: 146364022 },
    "9" => ContigMeta { ord: 8, len: 141213431 },
    "10" => ContigMeta { ord: 9, len: 135534747 },
    "11" => ContigMeta { ord: 10, len: 135006516 },
    "12" => ContigMeta { ord: 11, len: 133851895 },
    "13" => ContigMeta { ord: 12, len: 115169878 },
    "14" => ContigMeta { ord: 13, len: 107349540 },
    "15" => ContigMeta { ord: 14, len: 102531392 },
    "16" => ContigMeta { ord: 15, len: 90354753 },
    "17" => ContigMeta { ord: 16, len: 81195210 },
    "18" => ContigMeta { ord: 17, len: 78077248 },
    "19" => ContigMeta { ord: 18, len: 59128983 },
    "20" => ContigMeta { ord: 19, len: 63025520 },
    "21" => ContigMeta { ord: 20, len: 48129895 },
    "22" => ContigMeta { ord: 21, len: 51304566 },
    "X" => ContigMeta { ord: 22, len: 155270560 },
    "Y" => ContigMeta { ord: 23, len: 59373566 },
    "MT" => ContigMeta { ord: 24, len: 16569 },
    "GL000207.1" => ContigMeta { ord: 25, len: 4262 },
    "GL000226.1" => ContigMeta { ord: 26, len: 15008 },
    "GL000229.1" => ContigMeta { ord: 27, len: 19913 },
    "GL000231.1" => ContigMeta { ord: 28, len: 27386 },
    "GL000210.1" => ContigMeta { ord: 29, len: 27682 },
    "GL000239.1" => ContigMeta { ord: 30, len: 33824 },
    "GL000235.1" => ContigMeta { ord: 31, len: 34474 },
    "GL000201.1" => ContigMeta { ord: 32, len: 36148 },
    "GL000247.1" => ContigMeta { ord: 33, len: 36422 },
    "GL000245.1" => ContigMeta { ord: 34, len: 36651 },
    "GL000197.1" => ContigMeta { ord: 35, len: 37175 },
    "GL000203.1" => ContigMeta { ord: 36, len: 37498 },
    "GL000246.1" => ContigMeta { ord: 37, len: 38154 },
    "GL000249.1" => ContigMeta { ord: 38, len: 38502 },
    "GL000196.1" => ContigMeta { ord: 39, len: 38914 },
    "GL000248.1" => ContigMeta { ord: 40, len: 39786 },
    "GL000244.1" => ContigMeta { ord: 41, len: 39929 },
    "GL000238.1" => ContigMeta { ord: 42, len: 39939 },
    "GL000202.1" => ContigMeta { ord: 43, len: 40103 },
    "GL000234.1" => ContigMeta { ord: 44, len: 40531 },
    "GL000232.1" => ContigMeta { ord: 45, len: 40652 },
    "GL000206.1" => ContigMeta { ord: 46, len: 41001 },
    "GL000240.1" => ContigMeta { ord: 47, len: 41933 },
    "GL000236.1" => ContigMeta { ord: 48, len: 41934 },
    "GL000241.1" => ContigMeta { ord: 49, len: 42152 },
    "GL000243.1" => ContigMeta { ord: 50, len: 43341 },
    "GL000242.1" => ContigMeta { ord: 51, len: 43523 },
    "GL000230.1" => ContigMeta { ord: 52, len: 43691 },
    "GL000237.1" => ContigMeta { ord: 53, len: 45867 },
    "GL000233.1" => ContigMeta { ord: 54, len: 45941 },
    "GL000204.1" => ContigMeta { ord: 55, len: 81310 },
    "GL000198.1" => ContigMeta { ord: 56, len: 90085 },
    "GL000208.1" => ContigMeta { ord: 57, len: 92689 },
    "GL000191.1" => ContigMeta { ord: 58, len: 106433 },
    "GL000227.1" => ContigMeta { ord: 59, len: 128374 },
    "GL000228.1" => ContigMeta { ord: 60, len: 129120 },
    "GL000214.1" => ContigMeta { ord: 61, len: 137718 },
    "GL000221.1" => ContigMeta { ord: 62, len: 155397 },
    "GL000209.1" => ContigMeta { ord: 63, len: 159169 },
    "GL000218.1" => ContigMeta { ord: 64, len: 161147 },
    "GL000220.1" => ContigMeta { ord: 65, len: 161802 },
    "GL000213.1" => ContigMeta { ord: 66, len: 164239 },
    "GL000211.1" => ContigMeta { ord: 67, len: 166566 },
    "GL000199.1" => ContigMeta { ord: 68, len: 169874 },
    "GL000217.1" => ContigMeta { ord: 69, len: 172149 },
    "GL000216.1" => ContigMeta { ord: 70, len: 172294 },
    "GL000215.1" => ContigMeta { ord: 71, len: 172545 },
    "GL000205.1" => ContigMeta { ord: 72, len: 174588 },
    "GL000219.1" => ContigMeta { ord: 73, len: 179198 },
    "GL000224.1" => ContigMeta { ord: 74, len: 179693 },
    "GL000223.1" => ContigMeta { ord: 75, len: 180455 },
    "GL000195.1" => ContigMeta { ord: 76, len: 182896 },
    "GL000212.1" => ContigMeta { ord: 77, len: 186858 },
    "GL000222.1" => ContigMeta { ord: 78, len: 186861 },
    "GL000200.1" => ContigMeta { ord: 79, len: 187035 },
    "GL000193.1" => ContigMeta { ord: 80, len: 189789 },
    "GL000194.1" => ContigMeta { ord: 81, len: 191469 },
    "GL000225.1" => ContigMeta { ord: 82, len: 211173 },
    "GL000192.1" => ContigMeta { ord: 83, len: 547496 },
};

#[cfg(test)]
mod tests {
    use utile::resource::{RawResource, RawResourceExt};

    use crate::resource::HailCommonResource;

    use super::ContigMeta;

    #[test]
    #[ignore]
    fn gen_grch37_contig_meta() {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("reqwest", log::LevelFilter::Info)
            .filter_module("hyper_util", log::LevelFilter::Info)
            .init();

        let resource = HailCommonResource::old_grch37_reference_genome()
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached()
            .unwrap()
            .decompressed()
            .buffered();

        let mut reader = noodles::fasta::Reader::new(resource.read().unwrap());

        for (ord, record) in reader.records().enumerate() {
            let record = record.unwrap();
            let definition = record.definition().to_string();
            let definition = definition.strip_prefix('>').unwrap();

            let (name, _description) = definition.split_once(' ').unwrap();

            let meta = ContigMeta {
                ord,
                len: record.sequence().len() as u64,
            };

            println!("{name:?} => {meta:?},");
        }
    }

    #[test]
    #[ignore]
    fn test_grch37_contig_meta() {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("reqwest", log::LevelFilter::Info)
            .filter_module("hyper_util", log::LevelFilter::Info)
            .init();

        let resource = HailCommonResource::old_grch37_reference_genome()
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached()
            .unwrap()
            .decompressed()
            .buffered();

        let mut reader = noodles::fasta::Reader::new(resource.read().unwrap());

        for (ord, record) in reader.records().enumerate() {
            let record = record.unwrap();
            let definition = record.definition().to_string();
            let definition = definition.strip_prefix('>').unwrap();

            let (name, _description) = definition.split_once(' ').unwrap();

            let meta = ContigMeta {
                ord,
                len: record.sequence().len() as u64,
            };

            assert_eq!(super::META[name], meta);
        }
    }
}
