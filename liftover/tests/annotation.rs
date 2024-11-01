const CHAIN: &str =
    "chain 12789731909 chrX 149249818 + 400030 149249818 chrX 153692391 + 498000 153692391 8
2674	0	1345
213	0	202
363	2	2
16	1	1
914	1	1
45	2	0
";

/// Just a small helper to annotate the a chain with resolved offsets.
#[ignore]
#[test]
fn annotate_chain() {
    let mut output = String::new();
    let mut target_sum = 0;
    let mut query_sum = 0;

    for (i, line) in CHAIN.lines().enumerate() {
        if i == 0 {
            // Parse the chain header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 13 {
                let [_chain, _score, _t_name, _t_size, _t_strand, t_start, _t_end, _q_name, _q_size, _q_strand, q_start, _q_end, _id] =
                    parts[..]
                else {
                    panic!("Unexpected header format");
                };
                target_sum = t_start.parse().unwrap();
                query_sum = q_start.parse().unwrap();
            }
            // If header doesn't have expected parts, copy as is
            output.push_str(&format!("{}\n", line));
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let size: i64 = parts[0].parse().unwrap();
            let dt: i64 = parts[1].parse().unwrap();
            let dq: i64 = parts[2].parse().unwrap();

            target_sum += size + dt;
            query_sum += size + dq;

            output.push_str(&format!(
                "{size}\t{dt}\t{dq}\t// ({target_sum}, {query_sum})\n",
            ));
        } else {
            // If the line doesn't have 3 parts, copy it as is
            output.push_str(&format!("{}\n", line));
        }
    }

    println!("{output}");
}
