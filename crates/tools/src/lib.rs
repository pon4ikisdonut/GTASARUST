//! Shared helpers for GTASARUST CLI & GUI developer tools.

use anyhow::Result;
use gtasarust_asset_pipeline::{InstallationReport, inspect_installation};
use std::path::Path;

pub fn inspect_and_summarize(game_dir: &Path) -> Result<InstallationReport> {
    Ok(inspect_installation(game_dir)?)
}
