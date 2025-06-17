use std::{ffi::OsStr, fs::File, io::Write, path::Path};

use biocore::location::{GenomePosition, GenomeRange};

use super::{FailureReason, PositionFailureReason, UcscLiftoverSettings};

pub async fn liftover_human_snps(
    locations: &[GenomePosition],
    chain_file: impl AsRef<Path>,
    liftover_command: impl AsRef<OsStr>,
    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomePosition>, PositionFailureReason>>> {
    Ok(liftover(
        &locations
            .iter()
            .cloned()
            .map(GenomeRange::from)
            .collect::<Vec<_>>(),
        chain_file.as_ref(),
        liftover_command,
        settings,
    )
    .await?
    .into_iter()
    .enumerate()
    .map(|(i, v)| super::recover_positions(&locations[i], v?))
    .collect())
}
pub async fn liftover_snps(
    locations: &[GenomePosition],
    chain_file: impl AsRef<Path>,
    liftover_command: impl AsRef<OsStr>,
    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomePosition>, PositionFailureReason>>> {
    Ok(liftover(
        &locations
            .iter()
            .cloned()
            .map(GenomeRange::from)
            .collect::<Vec<_>>(),
        chain_file,
        liftover_command,
        settings,
    )
    .await?
    .into_iter()
    .enumerate()
    .map(|(i, v)| super::recover_positions(&locations[i], v?))
    .collect())
}

pub async fn liftover_human(
    locs: &[GenomeRange],
    chain_file: impl AsRef<Path>,
    liftover_command: impl AsRef<OsStr>,
    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomeRange>, FailureReason>>> {
    liftover(locs, chain_file, liftover_command, settings).await
}

pub async fn liftover(
    locations: &[GenomeRange],
    chain_file: impl AsRef<Path>,
    liftover_command: impl AsRef<OsStr>,
    settings: UcscLiftoverSettings,
) -> std::io::Result<Vec<Result<Vec<GenomeRange>, FailureReason>>> {
    let input = tempfile::Builder::new().tempfile()?;
    let output = tempfile::Builder::new().tempfile()?;
    let unmapped = tempfile::Builder::new().tempfile()?;

    {
        let mut writer = File::create(input.path())?;
        for loc in locations {
            writeln!(writer, "{} {} {}", loc.name, loc.at.start, loc.at.end)?;
        }
    }

    let mut command = utile::wsl::new_wsl_command(liftover_command);

    let settings = settings.preprocess();

    command
        .arg(utile::wsl::to_wsl_path(input.path())?)
        .arg(utile::wsl::to_wsl_path(chain_file)?)
        .arg(utile::wsl::to_wsl_path(output.path())?)
        .arg(utile::wsl::to_wsl_path(unmapped.path())?)
        .arg(format!("-minMatch={}", settings.min_match))
        .arg("-multiple")
        .arg(format!("-minSizeQ={}", settings.min_query))
        .arg(format!("-minChainT={}", settings.min_chain))
        .arg(format!("-minBlocks={}", settings.min_blocks));

    let result = command
        .output()
        .unwrap_or_else(|e| panic!("Failed to run command: {command:?}\n{e:?}"));

    if !result.status.success() {
        return Err(std::io::Error::other(format!(
            "liftOver command failed:\n{result:?}"
        )));
    }

    let success_results = super::parse_success_file(&std::fs::read_to_string(output.path())?)?;

    let failure_results = super::parse_failure_file(&std::fs::read_to_string(unmapped.path())?)?;

    super::combine_success_and_failure(locations, Some(success_results), Some(failure_results))
}
