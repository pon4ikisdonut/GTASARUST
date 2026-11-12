use anyhow::Result;
use clap::Parser;
use gtasarust_engine::init_logging;
use gtasarust_gta_formats::{
    img::ImgArchive,
    rw::{DffClump, TxdArchive},
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gtasarust-model-viewer",
    about = "Inspect RenderWare DFF/TXD models from files or IMG archives"
)]
struct Cli {
    /// Path to a .dff / .txd file, or an .img archive when --entry is specified.
    #[arg(long)]
    path: PathBuf,
    /// Optional entry inside an IMG archive (e.g. infernus.dff).
    #[arg(long)]
    entry: Option<String>,
}

fn main() -> Result<()> {
    init_logging(false);
    let cli = Cli::parse();
    let bytes = if let Some(entry_name) = &cli.entry {
        let img = ImgArchive::open(&cli.path)?;
        img.read_entry(entry_name)?
    } else {
        std::fs::read(&cli.path)?
    };

    let target_name = cli
        .entry
        .as_deref()
        .or_else(|| cli.path.file_name().and_then(|s| s.to_str()))
        .unwrap_or("")
        .to_ascii_lowercase();

    if target_name.ends_with(".txd") {
        let txd = TxdArchive::parse(&bytes)?;
        println!("TXD textures: {}", txd.textures.len());
        for t in &txd.textures {
            println!(" - {} ({}x{})", t.name, t.width, t.height);
        }
    } else {
        let clump = DffClump::parse(&bytes)?;
        println!(
            "DFF rw_version=0x{:X}, frames={}, geometries={}, atomics={}",
            clump.rw_version,
            clump.frames.len(),
            clump.geometries.len(),
            clump.atomics.len()
        );
        for (i, g) in clump.geometries.iter().enumerate() {
            println!(
                "  Geometry[{i}]: verts={}, normals={}, uvs={}, tris={}, splits={}, materials={}",
                g.positions.len(),
                g.normals.len(),
                g.uvs.len(),
                g.triangles.len(),
                g.splits.len(),
                g.materials.len(),
            );
        }
    }
    Ok(())
}
