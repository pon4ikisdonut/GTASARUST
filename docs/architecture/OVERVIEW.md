# GTASARUST Engine Architecture

## Design Principles

1. **Clean Separation of Concerns**: Engine primitives (`engine`, `renderer`, `physics`, `audio`, `input`) contain no hardcoded San Andreas story logic. Clean-room format parsers (`gta_formats`) are completely decoupled from GPU/audio backends so they run headless in CLI tools and tests.
2. **Coordinate System & Scale**: GTA: San Andreas uses a right-handed Z-up world coordinate system spanning approximately `[-3000, +3000]` meters on X and Y. `gtasarust-renderer` and `gtasarust-physics` preserve SA's native Z-up world coordinates (`+X` East, `+Y` North, `+Z` Up) inside simulation and transform consistently into clip space in the camera matrix, avoiding coordinate-flipping bugs across IPL placements, COL bounds, and `handling.cfg` centers of mass!
3. **Cross-Platform Filesystem Abstraction**: `gtasarust-engine::vfs::GameDataRoot` handles `\` vs `/` normalization and case-insensitive path lookups so `gta.dat`, `default.dat`, and `.ide`/`.ipl` directives resolve identically on Windows and Linux.
4. **Fixed-Timestep Simulation**: Core physics, AI, vehicle dynamics, and mission scripting tick at a deterministic fixed step (`60 Hz` / `dt = 1/60s`, clamped accumulator) decoupled from render frame rate.

## Crate Dependency Graph

```text
gtasarust-game
 ├── gtasarust-engine (config, vfs, logging, lifecycle)
 ├── gtasarust-gta-formats (IMG, DFF, TXD, COL, IDE, IPL, GXT, DAT, SCM, IFP)
 ├── gtasarust-asset-pipeline (batch extract, convert, manifest, verify)
 ├── gtasarust-renderer (wgpu, WGSL, camera, sky/timecycle, LOD, instancing)
 ├── gtasarust-physics (rapier3d colliders, character & vehicle raycasts)
 ├── gtasarust-audio (kira mixer, SFX, radio)
 ├── gtasarust-input (winit + gilrs action mapping)
 ├── gtasarust-world (gta.dat loader, sectors, streaming, zones, water, paths)
 ├── gtasarust-entities (player, peds, pickups, props)
 ├── gtasarust-vehicles (handling.cfg, vehicle classes, seats, damage)
 ├── gtasarust-weapons (weapon.dat, melee/firearms/explosives)
 ├── gtasarust-ai (peds, traffic, police/wanted)
 ├── gtasarust-animation (IFP skeletons & state machine)
 ├── gtasarust-scripting (SCM bytecode VM)
 ├── gtasarust-missions (triggers, checkpoints, campaign)
 ├── gtasarust-localization (GXT multi-language tables)
 ├── gtasarust-ui (SA HUD, radar, menus, egui inspector)
 └── gtasarust-save-system (versioned saves, checksums, migrations)
```
