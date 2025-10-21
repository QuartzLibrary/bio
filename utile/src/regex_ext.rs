use std::{iter, ops::Range};

use regex::Regex;

/// Returns an iterator over all overlapping matches of a regex in a haystack.
pub fn find_iter_overlapping(regex: &Regex, haystack: &str) -> impl Iterator<Item = Range<u64>> {
    let mut offset = 0;
    iter::from_fn(move || {
        let found = regex.find(haystack.get(offset..)?)?;

        let start = offset + found.start();
        let end = offset + found.end();

        offset += found.start();
        offset += 1;

        Some(start as u64..end as u64)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    ///                                     .GG
    ///                                    .GG
    ///                          .GG      .GG                                                        .GG   .GG            .GG
    const TEST_SEQUENCE: &str = "GGGATCGCCAGGGGTCGTAATCGATCGTAATCGATCGTAATCGATCGTAATCGATCGTAATCGATCGTAGGATCCGG(AATT/AGCG)CCGGATCCTAGCGAGATCCTAGCGAGATCCTAGCGAGATCCTAGCGAGATCCTAGCGAGATCCTAGCGAT";

    #[test]
    fn test_pams() {
        let regex = Regex::new(".GG").unwrap();
        let pams = find_iter_overlapping(&regex, TEST_SEQUENCE);
        assert_eq!(
            pams.collect::<Vec<_>>(),
            vec![0..3, 9..12, 10..13, 11..14, 68..71, 74..77, 89..92]
        );
    }
}
