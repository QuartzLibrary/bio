use std::ops::Range;

use regex::{Regex, RegexBuilder};
use reqwest::Client;

use biocore::location::{GenomePosition, GenomeRange};

use crate::sources::UcscHG;

use super::{FailureReason, PositionFailureReason, UcscLiftoverSettings};

pub async fn liftover_human_snps(
    client: &Client,

    from_db: UcscHG,

    locations: &[GenomePosition],

    to_db: UcscHG,

    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomePosition>, PositionFailureReason>>> {
    Ok(liftover(
        client,
        "Human",
        from_db.name(),
        &locations
            .iter()
            .cloned()
            .map(GenomeRange::from)
            .collect::<Vec<_>>(),
        "Human",
        to_db.name(),
        settings,
    )
    .await?
    .into_iter()
    .enumerate()
    .map(|(i, v)| super::recover_positions(&locations[i], v?))
    .collect())
}

pub async fn liftover_snps(
    client: &Client,

    from_org: &str,
    from_db: &str,

    locations: &[GenomePosition],

    to_org: &str,
    to_db: &str,

    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomePosition>, PositionFailureReason>>> {
    Ok(liftover(
        client,
        from_org,
        from_db,
        &locations
            .iter()
            .cloned()
            .map(GenomeRange::from)
            .collect::<Vec<_>>(),
        to_org,
        to_db,
        settings,
    )
    .await?
    .into_iter()
    .enumerate()
    .map(|(i, v)| super::recover_positions(&locations[i], v?))
    .collect())
}

pub async fn liftover_human(
    client: &Client,

    from_db: UcscHG,

    locs: &[GenomeRange],

    to_db: UcscHG,

    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomeRange>, FailureReason>>> {
    liftover(
        client,
        "Human",
        from_db.name(),
        locs,
        "Human",
        to_db.name(),
        settings,
    )
    .await
}

pub async fn liftover(
    client: &Client,

    from_org: &str,
    from_db: &str,

    locations: &[GenomeRange],

    to_org: &str,
    to_db: &str,

    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomeRange>, FailureReason>>> {
    // TODO: try and simplify to use normal form handlers.

    let id: u128 = rand::random();
    let res = client
        .post("https://genome.ucsc.edu/cgi-bin/hgLiftOver")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary=---------------------------{id}"),
        )
        .body(build_payload(
            from_org, from_db, locations, to_org, to_db, settings, id,
        ))
        .send()
        .await
        .map_err(utile::io::reqwest_error)?
        .text()
        .await
        .map_err(utile::io::reqwest_error)?;

    let success = {
        let file = get_success_result(client, &res)
            .await
            .map_err(utile::io::reqwest_error)?;
        match file {
            Some(file) => Some(super::parse_success_file(&file)?),
            None => None,
        }
    };
    let failure = {
        let file = get_failure_result(client, &res)
            .await
            .map_err(utile::io::reqwest_error)?;
        match file {
            Some(file) => Some(super::parse_failure_file(&file)?),
            None => None,
        }
    };

    super::combine_success_and_failure(locations, success, failure)
}

async fn get_success_result(client: &Client, res: &str) -> reqwest::Result<Option<String>> {
    // Extract the result file path
    let re = Regex::new(r"(\.\./trash[^\ >]+\.bed)").unwrap();
    let Some(file_path) = re.captures(res).and_then(|cap| cap.get(1)) else {
        return Ok(None);
    };
    let file_path = file_path.as_str();

    // GET request to fetch the result
    let res = client
        .get(format!("https://genome.ucsc.edu/cgi-bin/{file_path}"))
        .send()
        .await?
        .text()
        .await?;

    Ok(Some(res))
}
async fn get_failure_result(client: &Client, res: &str) -> reqwest::Result<Option<String>> {
    let re = Regex::new(r"(\.\./trash[^\ >]+\.err)").unwrap();
    let Some(file_path) = re.captures(res).and_then(|cap| cap.get(1)) else {
        return Ok(None);
    };
    let file_path = file_path.as_str();

    let res = client
        .get(format!("https://genome.ucsc.edu/cgi-bin/{file_path}"))
        .send()
        .await?
        .text()
        .await?;

    Ok(Some(res))
}

#[allow(dead_code)]
async fn get_rsid(client: &Client) -> std::io::Result<String> {
    const RSDI_REGEX: &str = r#"<input type=(?:(?:['"]hidden['"])|(?:hidden)) name=(?:(?:['"]hgsid['"])|(?:hgsid)) value=['"]([^'"]+)[^>]"#;

    let res = client
        .get("https://genome.ucsc.edu/cgi-bin/hgLiftOver")
        .send()
        .await
        .map_err(utile::io::reqwest_error)?
        .text()
        .await
        .map_err(utile::io::reqwest_error)?;

    Ok(RegexBuilder::new(RSDI_REGEX)
        .case_insensitive(true)
        .build()
        .unwrap()
        .captures(&res)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Failed to extract hgsid",
        ))?
        .to_owned())
}

fn build_payload(
    from_org: &str,
    from_db: &str,
    locations: &[GenomeRange],
    to_org: &str,
    to_db: &str,
    settings: UcscLiftoverSettings,
    id: u128,
) -> String {
    let locations = {
        let mut l = String::new();
        for GenomeRange {
            name,
            at: Range { start, end },
        } in locations
        {
            l.push_str(&format!("{name} {start} {end}\n"));
        }

        l
    };

    let UcscLiftoverSettings {
        min_match,
        // keep_original_positions,
        // multi_region_allowed,
        min_query,
        min_chain,
        min_blocks,
        // is_thick_fudge_set,
    } = settings.preprocess();

    format!(
        r##"-----------------------------{id}
Content-Disposition: form-data; name="hgsid"

2367095561_CoHtZHJcvc24pT3opYDrrv33ChBZ
-----------------------------{id}
Content-Disposition: form-data; name="hglft_fromOrg"

{from_org}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_fromDb"

{from_db}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_toOrg"

{to_org}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_toDb"

{to_db}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_minMatch"

{min_match}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_extranameinfo"

on
-----------------------------{id}
Content-Disposition: form-data; name="boolshad.hglft_extranameinfo"

0
-----------------------------{id}
Content-Disposition: form-data; name="hglft_multiple"

on
-----------------------------{id}
Content-Disposition: form-data; name="boolshad.hglft_multiple"

0
-----------------------------{id}
Content-Disposition: form-data; name="hglft_minSizeQ"

{min_query}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_minChainT"

{min_chain}
-----------------------------{id}
Content-Disposition: form-data; name="hglft_minBlocks"

{min_blocks}
-----------------------------{id}
Content-Disposition: form-data; name="boolshad.hglft_fudgeThick"

0
-----------------------------{id}
Content-Disposition: form-data; name="hglft_userData"

{locations}
-----------------------------{id}
Content-Disposition: form-data; name="Submit"

Submit
-----------------------------{id}
Content-Disposition: form-data; name="hglft_dataFile"; filename=""
Content-Type: application/octet-stream


-----------------------------{id}
Content-Disposition: form-data; name="hglft_doRefreshOnly"

0
-----------------------------{id}
Content-Disposition: form-data; name="hglft_lastChain"

{from_db}.{to_db}
-----------------------------{id}
Content-Disposition: form-data; name="hgsid"

2367095561_CoHtZHJcvc24pT3opYDrrv33ChBZ
-----------------------------{id}--
"##
    )
}
