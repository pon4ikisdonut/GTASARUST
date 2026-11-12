# Changelog

All notable changes to **GTASARUST** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project governance, documentation, and issue/PR templates.
- 24-crate Rust 2024 workspace covering engine, `wgpu` rendering, `rapier3d` physics, `kira` audio, `winit`/`gilrs` input, world streaming, entities, vehicles, weapons, AI, missions, scripting, animation, `egui` UI, save system, asset pipeline, GTA formats, localization, `gtasarust` binary, and 4 CLI inspection/conversion tools (`asset-extractor`, `asset-converter`, `map-inspector`, `model-viewer`).
- Clean-room GTA:SA readers validated against PC DVD v1.00 (`E:\gtasa`): 8 IMG archives, 15,334 DFFs, 3,983 TXDs, 257 COLs, 14,344 IDE objects, 45,837 text+binary IPL placements, 16,588 GXT strings, 210 `handling.cfg` entries, `timecyc.dat`, `water.dat`, `weapon.dat`, `ped.ifp`, and `main.scm`.
- Cross-platform CI workflow for Windows x86_64 (`x86_64-pc-windows-msvc`) and Linux x86_64 (`x86_64-unknown-linux-gnu`).

### Fixed
- Enabled `egui/default_fonts` so HUD, main menu, pause menu, and debug inspector text renders properly at runtime.
- Handled 0-byte audio stream placeholders and `VER2` IMG streaming sector fallback safely.
