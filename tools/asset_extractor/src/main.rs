use anyhow::Result;
use clap::{Parser, Subcommand};
use gtasarust_asset_pipeline::{BatchConvertOptions, convert_batch, inspect_installation};
use gtasarust_engine::init_logging;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gtasarust-asset-extractor",
    about = "Inspect and extract authorized GTA:SA archives"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect an authorized GTA:SA directory and print validated asset statistics.
    Inspect {
        #[arg(long, env = "GTASA_DATA_DIR")]
        game_dir: PathBuf,
        #[arg(long)]
        manifest_out: Option<PathBuf>,
    },
    /// Batch extract & convert IMG entries into validated intermediate JSON/PNG files.
    Extract {
        #[arg(long, env = "GTASA_DATA_DIR")]
        game_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long)]
        png: bool,
    },
}

fn main() -> Result<()> {
    init_logging(false);
    let cli = Cli::parse();
    match cli.cmd {
        Command::Inspect {
            game_dir,
            manifest_out,
        } => {
            let report = inspect_installation(&game_dir)?;
            let json = serde_json::to_string_pretty(&report)?;
            if let Some(out) = manifest_out {
                if let Some(p) = out.parent() {
                    std::fs::create_dir_all(p)?;
                }
                std::fs::write(&out, &json)?;
            }
            println!("{json}");
        }
        Command::Extract {
            game_dir,
            out,
            limit,
            prefix,
            png,
        } => {
            let report = convert_batch(&BatchConvertOptions {
                game_dir,
                output_dir: out,
                max_entries: limit,
                export_png_textures: png,
                filter_prefix: prefix,
            })?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}
