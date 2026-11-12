use anyhow::Result;
use clap::Parser;
use gtasarust_engine::{GameDataRoot, init_logging};
use gtasarust_world::WorldManager;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gtasarust-map-inspector",
    about = "Inspect San Andreas IDE/IPL world streaming sectors"
)]
struct Cli {
    #[arg(long, env = "GTASA_DATA_DIR")]
    game_dir: PathBuf,
    #[arg(long, default_value_t = false)]
    include_binary_ipls: bool,
}

fn main() -> Result<()> {
    init_logging(false);
    let cli = Cli::parse();
    let vfs = GameDataRoot::open(&cli.game_dir)?;
    let world = WorldManager::load_from_game_dir(&vfs, cli.include_binary_ipls);
    println!(
        "Objects: {}, Vehicles: {}, Peds: {}, Sectors: {}, Total Placements: {}, Zones: {}, Water Quads: {}",
        world.objects.len(),
        world.vehicles.len(),
        world.peds.len(),
        world.sectors.len(),
        world.total_placements(),
        world.zones.len(),
        world.water_quads.len(),
    );
    Ok(())
}
