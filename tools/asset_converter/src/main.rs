use anyhow::Result;
use clap::Parser;
use gtasarust_asset_pipeline::{BatchConvertOptions, convert_batch};
use gtasarust_engine::init_logging;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gtasarust-asset-converter",
    about = "Convert GTA:SA assets into GTASARUST formats"
)]
struct Cli {
    #[arg(long, env = "GTASA_DATA_DIR")]
    game_dir: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long, default_value_t = true)]
    export_png: bool,
}

fn main() -> Result<()> {
    init_logging(false);
    let cli = Cli::parse();
    let report = convert_batch(&BatchConvertOptions {
        game_dir: cli.game_dir,
        output_dir: cli.out,
        max_entries: cli.limit,
        export_png_textures: cli.export_png,
        filter_prefix: None,
    })?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
