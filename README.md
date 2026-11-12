# GTASARUST

An AI-led, open-source Rust recreation of **Grand Theft Auto: San Andreas**, featuring a complete game world, gameplay systems, missions, vehicles, and cross-platform support.

> **Autonomous AI Development Notice**
>
> This project is developed and maintained by an AI-driven development team. AI agents are responsible for architecture, programming, implementation, testing, documentation, and continuous development.

> **Legal & Affiliation Disclaimer**
>
> GTASARUST is an independent, clean-room open-source engine and gameplay recreation project. It is **not** affiliated with, sponsored by, or endorsed by Rockstar Games or Take-Two Interactive Software, Inc.
>
> **No copyrighted game assets are included in this repository.** To run GTASARUST with original San Andreas content, you must supply your own authorized, legally obtained PC copy of *Grand Theft Auto: San Andreas* (e.g., via `GTASA_DATA_DIR` or `--game-dir`).

---

## Project Goals

GTASARUST aims for a complete, faithful, cross-platform (`x86_64` Windows & Linux) recreation of *Grand Theft Auto: San Andreas* written in Rust:

1. **Original Asset Pipeline (`gta_formats`, `asset_pipeline`)** — Parse original IMG (`VER2`), DFF (RenderWare Clump/Geometry), TXD (RenderWare TexDictionary + DXT/RGBA raster decoding), COL (COL1/COL2/COL3/COL4), IDE, IPL (text & binary), DAT/CFG, IFP, GXT, SCM, and audio bank/stream formats, converting them into reproducible, validated internal assets with manifest verification.
2. **Engine & Rendering (`engine`, `renderer`, `physics`, `audio`, `input`)** — `wgpu` + WGSL renderer recreating the original timecycle, weather, water, LODs, and world atmosphere; `rapier3d` physics; `kira` audio mixing & radio; `winit` + `gilrs` input; `egui` developer tools + authentic in-game UI.
3. **Complete World (`world`)** — Streaming across Los Santos, San Fierro, Las Venturas, countryside, desert, interiors, water, and traffic/pedestrian path nodes without artificial loading walls.
4. **Gameplay, Vehicles, AI & Story (`entities`, `vehicles`, `weapons`, `ai`, `missions`, `scripting`, `save_system`)** — Distinct handling per vehicle class, player animation state machines, wanted levels & police escalation, shops, properties, gang territories, versioned save slots, and a full SCM-compatible mission runtime.

---

## Supported Platforms

GTASARUST targets **64-bit (`x86_64` / `AMD64`) ONLY**:

- **Windows 10 x64 / Windows 11 x64** (`x86_64-pc-windows-msvc`)
- **Linux x86_64** (`x86_64-unknown-linux-gnu` — Ubuntu, Debian, Fedora, Arch Linux)

32-bit builds are explicitly unsupported.

---

## Repository Structure

```text
GTASARUST/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── engine/                 # Application lifecycle, logging, config, platform paths, ECS/scene core
│   ├── renderer/               # wgpu + WGSL rendering pipeline, camera, sky/timecycle, LODs
│   ├── physics/                # rapier3d integration, world & vehicle collision
│   ├── audio/                  # kira audio engine, SFX banks, radio streams, mixing
│   ├── input/                  # winit keyboard/mouse + gilrs gamepad abstraction
│   ├── world/                  # Spatial partitioning, streaming sectors, IDE/IPL/zones/paths
│   ├── entities/               # Player, pedestrians, pickups, world objects
│   ├── vehicles/               # Vehicle classes, handling.cfg physics, damage, entry/exit
│   ├── weapons/                # Melee, firearms, projectiles, explosions, weapon.dat
│   ├── ai/                     # Pedestrian AI, traffic lanes, police & wanted escalation
│   ├── missions/               # Campaign state, triggers, checkpoints, objectives
│   ├── scripting/              # SCM bytecode & mission script VM
│   ├── animation/              # IFP skeletal animation & character state machine
│   ├── ui/                     # Authentic HUD, radar, menus, egui dev overlay
│   ├── save_system/            # Versioned save files, migrations, corruption checks
│   ├── asset_pipeline/         # Reproducible batch extractor, converter, manifest & validator
│   ├── gta_formats/            # Clean-room parsers for IMG, DFF, TXD, COL, IDE, IPL, GXT, SCM, etc.
│   ├── localization/           # GXT text tables and multi-language string lookup
│   ├── game/                   # Main GTASARUST playable binary
│   └── tools/                  # Shared CLI tooling utilities
├── tools/
│   ├── asset_extractor/        # Batch asset extraction & inspection CLI
│   ├── asset_converter/        # Asset conversion & validation CLI
│   ├── map_inspector/          # World/IPL/IDE diagnostic inspector
│   └── model_viewer/           # Standalone DFF/TXD/COL viewer
├── assets/                     # Engine shaders, fonts, UI definitions & manifests (no copyrighted assets)
├── data/                       # Engine-owned configuration & schemas
├── tests/                      # Workspace integration, gameplay, and asset pipeline tests
└── docs/                       # Architecture, file formats, gameplay, and compatibility docs
```

---

## Quick Start

### Prerequisites

- **Rust** stable toolchain (`x86_64`)
- **Windows**: Visual Studio Build Tools (MSVC) & Vulkan/DX12-capable GPU drivers
- **Linux**: `gcc`/`clang`, `pkg-config`, Vulkan/Mesa drivers, X11/Wayland dev headers (`libxkbcommon-dev`, `libwayland-dev`, `libx11-dev`, `libasound2-dev`, `libudev-dev`)
- A legally obtained PC installation of *Grand Theft Auto: San Andreas* (for original game assets)

### Build & Test

```bash
# Run full workspace tests
cargo test --workspace

# Build Windows x64 release
cargo build --release --target x86_64-pc-windows-msvc

# Build Linux x64 release
cargo build --release --target x86_64-unknown-linux-gnu
```

### Running

```bash
# Point GTASARUST to your authorized GTA: San Andreas directory
cargo run -p gtasarust-game -- --game-dir "E:\gtasa"

# Or set the environment variable
export GTASA_DATA_DIR="/path/to/gtasa"   # Linux
set GTASA_DATA_DIR=E:\gtasa              # Windows
cargo run -p gtasarust-game
```

---

## Tracking & Documentation

- [Development Status](DEVELOPMENT_STATUS.md) — Live subsystem status (`Not started` / `In progress` / `Implemented` / `Tested` / `Verified`)
- [Feature Matrix](FEATURE_MATRIX.md) — Comparison between original *GTA: San Andreas* and GTASARUST
- [Compatibility Matrix](COMPATIBILITY.md) — OS, GPU driver, and game version compatibility
- [Known Bugs](BUGS.md) — Active issues and diagnostics
- [Changelog](CHANGELOG.md) — Release and milestone history
- [Contributing](CONTRIBUTING.md) — Guidelines for clean-room contributions and CI checks

---

## License

Original source code and documentation in this repository are licensed under the [MIT License](LICENSE). Original *Grand Theft Auto: San Andreas* assets remain the sole property of Rockstar Games / Take-Two Interactive and are never redistributed by this project.
