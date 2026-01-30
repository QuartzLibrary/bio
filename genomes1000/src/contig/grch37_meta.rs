#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct ContigMeta {
    pub ord: usize,
    pub len: u64,

    pub dna: Option<&'static str>,
    pub chromosome: Option<&'static str>,
    pub gi: Option<&'static str>,
    pub ref_: Option<&'static str>,
    pub supercontig: Option<&'static str>,
    pub rest: Option<&'static str>,
}

pub static META: phf::Map<&'static str, ContigMeta> = phf::phf_map! {
    "1" => ContigMeta { ord: 0, len: 249250621, dna: Some("chromosome"), chromosome: Some("GRCh37:1:1:249250621:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "2" => ContigMeta { ord: 1, len: 243199373, dna: Some("chromosome"), chromosome: Some("GRCh37:2:1:243199373:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "3" => ContigMeta { ord: 2, len: 198022430, dna: Some("chromosome"), chromosome: Some("GRCh37:3:1:198022430:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "4" => ContigMeta { ord: 3, len: 191154276, dna: Some("chromosome"), chromosome: Some("GRCh37:4:1:191154276:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "5" => ContigMeta { ord: 4, len: 180915260, dna: Some("chromosome"), chromosome: Some("GRCh37:5:1:180915260:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "6" => ContigMeta { ord: 5, len: 171115067, dna: Some("chromosome"), chromosome: Some("GRCh37:6:1:171115067:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "7" => ContigMeta { ord: 6, len: 159138663, dna: Some("chromosome"), chromosome: Some("GRCh37:7:1:159138663:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "8" => ContigMeta { ord: 7, len: 146364022, dna: Some("chromosome"), chromosome: Some("GRCh37:8:1:146364022:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "9" => ContigMeta { ord: 8, len: 141213431, dna: Some("chromosome"), chromosome: Some("GRCh37:9:1:141213431:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "10" => ContigMeta { ord: 9, len: 135534747, dna: Some("chromosome"), chromosome: Some("GRCh37:10:1:135534747:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "11" => ContigMeta { ord: 10, len: 135006516, dna: Some("chromosome"), chromosome: Some("GRCh37:11:1:135006516:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "12" => ContigMeta { ord: 11, len: 133851895, dna: Some("chromosome"), chromosome: Some("GRCh37:12:1:133851895:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "13" => ContigMeta { ord: 12, len: 115169878, dna: Some("chromosome"), chromosome: Some("GRCh37:13:1:115169878:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "14" => ContigMeta { ord: 13, len: 107349540, dna: Some("chromosome"), chromosome: Some("GRCh37:14:1:107349540:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "15" => ContigMeta { ord: 14, len: 102531392, dna: Some("chromosome"), chromosome: Some("GRCh37:15:1:102531392:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "16" => ContigMeta { ord: 15, len: 90354753, dna: Some("chromosome"), chromosome: Some("GRCh37:16:1:90354753:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "17" => ContigMeta { ord: 16, len: 81195210, dna: Some("chromosome"), chromosome: Some("GRCh37:17:1:81195210:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "18" => ContigMeta { ord: 17, len: 78077248, dna: Some("chromosome"), chromosome: Some("GRCh37:18:1:78077248:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "19" => ContigMeta { ord: 18, len: 59128983, dna: Some("chromosome"), chromosome: Some("GRCh37:19:1:59128983:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "20" => ContigMeta { ord: 19, len: 63025520, dna: Some("chromosome"), chromosome: Some("GRCh37:20:1:63025520:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "21" => ContigMeta { ord: 20, len: 48129895, dna: Some("chromosome"), chromosome: Some("GRCh37:21:1:48129895:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "22" => ContigMeta { ord: 21, len: 51304566, dna: Some("chromosome"), chromosome: Some("GRCh37:22:1:51304566:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "X" => ContigMeta { ord: 22, len: 155270560, dna: Some("chromosome"), chromosome: Some("GRCh37:X:1:155270560:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "Y" => ContigMeta { ord: 23, len: 59373566, dna: Some("chromosome"), chromosome: Some("GRCh37:Y:2649521:59034049:1"), gi: None, ref_: None, supercontig: None, rest: None },
    "MT" => ContigMeta { ord: 24, len: 16569, dna: None, chromosome: None, gi: Some("251831106"), ref_: Some("NC_012920.1"), supercontig: None, rest: Some("Homo sapiens mitochondrion, complete genome") },
    "GL000207.1" => ContigMeta { ord: 25, len: 4262, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000207.1:1:4262:1"), rest: None },
    "GL000226.1" => ContigMeta { ord: 26, len: 15008, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000226.1:1:15008:1"), rest: None },
    "GL000229.1" => ContigMeta { ord: 27, len: 19913, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000229.1:1:19913:1"), rest: None },
    "GL000231.1" => ContigMeta { ord: 28, len: 27386, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000231.1:1:27386:1"), rest: None },
    "GL000210.1" => ContigMeta { ord: 29, len: 27682, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000210.1:1:27682:1"), rest: None },
    "GL000239.1" => ContigMeta { ord: 30, len: 33824, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000239.1:1:33824:1"), rest: None },
    "GL000235.1" => ContigMeta { ord: 31, len: 34474, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000235.1:1:34474:1"), rest: None },
    "GL000201.1" => ContigMeta { ord: 32, len: 36148, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000201.1:1:36148:1"), rest: None },
    "GL000247.1" => ContigMeta { ord: 33, len: 36422, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000247.1:1:36422:1"), rest: None },
    "GL000245.1" => ContigMeta { ord: 34, len: 36651, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000245.1:1:36651:1"), rest: None },
    "GL000197.1" => ContigMeta { ord: 35, len: 37175, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000197.1:1:37175:1"), rest: None },
    "GL000203.1" => ContigMeta { ord: 36, len: 37498, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000203.1:1:37498:1"), rest: None },
    "GL000246.1" => ContigMeta { ord: 37, len: 38154, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000246.1:1:38154:1"), rest: None },
    "GL000249.1" => ContigMeta { ord: 38, len: 38502, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000249.1:1:38502:1"), rest: None },
    "GL000196.1" => ContigMeta { ord: 39, len: 38914, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000196.1:1:38914:1"), rest: None },
    "GL000248.1" => ContigMeta { ord: 40, len: 39786, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000248.1:1:39786:1"), rest: None },
    "GL000244.1" => ContigMeta { ord: 41, len: 39929, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000244.1:1:39929:1"), rest: None },
    "GL000238.1" => ContigMeta { ord: 42, len: 39939, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000238.1:1:39939:1"), rest: None },
    "GL000202.1" => ContigMeta { ord: 43, len: 40103, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000202.1:1:40103:1"), rest: None },
    "GL000234.1" => ContigMeta { ord: 44, len: 40531, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000234.1:1:40531:1"), rest: None },
    "GL000232.1" => ContigMeta { ord: 45, len: 40652, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000232.1:1:40652:1"), rest: None },
    "GL000206.1" => ContigMeta { ord: 46, len: 41001, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000206.1:1:41001:1"), rest: None },
    "GL000240.1" => ContigMeta { ord: 47, len: 41933, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000240.1:1:41933:1"), rest: None },
    "GL000236.1" => ContigMeta { ord: 48, len: 41934, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000236.1:1:41934:1"), rest: None },
    "GL000241.1" => ContigMeta { ord: 49, len: 42152, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000241.1:1:42152:1"), rest: None },
    "GL000243.1" => ContigMeta { ord: 50, len: 43341, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000243.1:1:43341:1"), rest: None },
    "GL000242.1" => ContigMeta { ord: 51, len: 43523, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000242.1:1:43523:1"), rest: None },
    "GL000230.1" => ContigMeta { ord: 52, len: 43691, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000230.1:1:43691:1"), rest: None },
    "GL000237.1" => ContigMeta { ord: 53, len: 45867, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000237.1:1:45867:1"), rest: None },
    "GL000233.1" => ContigMeta { ord: 54, len: 45941, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000233.1:1:45941:1"), rest: None },
    "GL000204.1" => ContigMeta { ord: 55, len: 81310, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000204.1:1:81310:1"), rest: None },
    "GL000198.1" => ContigMeta { ord: 56, len: 90085, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000198.1:1:90085:1"), rest: None },
    "GL000208.1" => ContigMeta { ord: 57, len: 92689, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000208.1:1:92689:1"), rest: None },
    "GL000191.1" => ContigMeta { ord: 58, len: 106433, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000191.1:1:106433:1"), rest: None },
    "GL000227.1" => ContigMeta { ord: 59, len: 128374, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000227.1:1:128374:1"), rest: None },
    "GL000228.1" => ContigMeta { ord: 60, len: 129120, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000228.1:1:129120:1"), rest: None },
    "GL000214.1" => ContigMeta { ord: 61, len: 137718, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000214.1:1:137718:1"), rest: None },
    "GL000221.1" => ContigMeta { ord: 62, len: 155397, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000221.1:1:155397:1"), rest: None },
    "GL000209.1" => ContigMeta { ord: 63, len: 159169, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000209.1:1:159169:1"), rest: None },
    "GL000218.1" => ContigMeta { ord: 64, len: 161147, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000218.1:1:161147:1"), rest: None },
    "GL000220.1" => ContigMeta { ord: 65, len: 161802, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000220.1:1:161802:1"), rest: None },
    "GL000213.1" => ContigMeta { ord: 66, len: 164239, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000213.1:1:164239:1"), rest: None },
    "GL000211.1" => ContigMeta { ord: 67, len: 166566, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000211.1:1:166566:1"), rest: None },
    "GL000199.1" => ContigMeta { ord: 68, len: 169874, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000199.1:1:169874:1"), rest: None },
    "GL000217.1" => ContigMeta { ord: 69, len: 172149, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000217.1:1:172149:1"), rest: None },
    "GL000216.1" => ContigMeta { ord: 70, len: 172294, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000216.1:1:172294:1"), rest: None },
    "GL000215.1" => ContigMeta { ord: 71, len: 172545, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000215.1:1:172545:1"), rest: None },
    "GL000205.1" => ContigMeta { ord: 72, len: 174588, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000205.1:1:174588:1"), rest: None },
    "GL000219.1" => ContigMeta { ord: 73, len: 179198, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000219.1:1:179198:1"), rest: None },
    "GL000224.1" => ContigMeta { ord: 74, len: 179693, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000224.1:1:179693:1"), rest: None },
    "GL000223.1" => ContigMeta { ord: 75, len: 180455, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000223.1:1:180455:1"), rest: None },
    "GL000195.1" => ContigMeta { ord: 76, len: 182896, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000195.1:1:182896:1"), rest: None },
    "GL000212.1" => ContigMeta { ord: 77, len: 186858, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000212.1:1:186858:1"), rest: None },
    "GL000222.1" => ContigMeta { ord: 78, len: 186861, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000222.1:1:186861:1"), rest: None },
    "GL000200.1" => ContigMeta { ord: 79, len: 187035, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000200.1:1:187035:1"), rest: None },
    "GL000193.1" => ContigMeta { ord: 80, len: 189789, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000193.1:1:189789:1"), rest: None },
    "GL000194.1" => ContigMeta { ord: 81, len: 191469, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000194.1:1:191469:1"), rest: None },
    "GL000225.1" => ContigMeta { ord: 82, len: 211173, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000225.1:1:211173:1"), rest: None },
    "GL000192.1" => ContigMeta { ord: 83, len: 547496, dna: Some("supercontig"), chromosome: None, gi: None, ref_: None, supercontig: Some(":GL000192.1:1:547496:1"), rest: None },
};

#[cfg(test)]
mod tests {
    use resource::{RawResource, RawResourceExt};

    use crate::source::Genomes1000Resource;

    use super::*;

    #[test]
    #[ignore]
    fn gen_grch37_contig_meta() {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("reqwest", log::LevelFilter::Info)
            .filter_module("hyper_util", log::LevelFilter::Info)
            .init();

        let resource = Genomes1000Resource::old_grch37_reference_genome()
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached()
            .unwrap()
            .decompressed()
            .buffered();

        let mut reader = noodles::fasta::io::Reader::new(resource.read().unwrap());

        for (ord, record) in reader.records().enumerate() {
            let record = record.unwrap();

            let (name, meta) = meta(ord, &record);

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

        let resource = Genomes1000Resource::old_grch37_reference_genome()
            .log_progress()
            .with_global_fs_cache()
            .ensure_cached()
            .unwrap()
            .decompressed()
            .buffered();

        let mut reader = noodles::fasta::io::Reader::new(resource.read().unwrap());

        for (ord, record) in reader.records().enumerate() {
            let record = record.unwrap();

            let (name, meta) = meta(ord, &record);

            assert_eq!(super::META[&name], meta);
        }
    }

    fn meta(ord: usize, record: &noodles::fasta::Record) -> (String, ContigMeta) {
        let definition = record.definition().to_string();
        let definition = definition.strip_prefix('>').unwrap();

        let (name, description) = definition.split_once(' ').unwrap();

        assert!(super::META.contains_key(name), "Missing contig: {name}");

        let meta = {
            let mut dna: Option<&'static str> = None;
            let mut chromosome: Option<&'static str> = None;
            let mut gi: Option<&'static str> = None;
            let mut supercontig: Option<&'static str> = None;
            let mut ref_: Option<&'static str> = None;
            let mut rest: Option<&'static str> = None;
            for value in &*description.split_whitespace().collect::<Vec<_>>() {
                if name == "MT" {
                    gi = Some("251831106");
                    ref_ = Some("NC_012920.1");
                    rest = Some("Homo sapiens mitochondrion, complete genome");
                } else {
                    match value.split_once(':').unwrap() {
                        ("dna", v) => dna = Some(v.to_string().leak()),
                        ("gi", v) => gi = Some(v.to_string().leak()),
                        ("chromosome", v) => chromosome = Some(v.to_string().leak()),
                        ("supercontig", v) => supercontig = Some(v.to_string().leak()),
                        ("ref", v) => ref_ = Some(v.to_string().leak()),
                        (p, _) => panic!("Invalid description: {description} (key: {p})"),
                    }
                }
            }
            ContigMeta {
                ord,
                len: record.sequence().len() as u64,
                dna,
                chromosome,
                gi,
                ref_,
                supercontig,
                rest,
            }
        };

        (name.to_owned(), meta)
    }
}
