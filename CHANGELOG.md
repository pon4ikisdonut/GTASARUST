# Changelog

All notable changes to **GTASARUST** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Full `ANP3` `ped.ifp` skeletal bone keyframe decoder (`gtasarust-gta-formats/ifp.rs`), `RW_SKIN_PLG` (`0x0116`) & `RW_HANIM_PLG` (`0x011E`) parsers (`rw.rs`), `IfpPoseLibrary` & `compute_skin_matrices` (`gtasarust-animation`), dynamic vertex buffer uploads/updates & `SkinnedVertexWeights` (`gtasarust-renderer`), and per-frame skinned CJ (`fam1`) + NPC (`fam2`/`fam3`/`bmydj`/`lapd1`/`hmost`) animation (`gtasarust-game`, `1,011` vertices verified moving on `fam1.dff` under `walk_civi`).
- `gta_int.img` binary IPL streaming (`50,935` placements / `1,457` sectors / `376` `ENEX` portals), floating yellow `ENEX` doorway cones (`CpuMeshData::marker_cone`), radar doorway blips & top-left help toast HUD (`gtasarust-ui`), interior `.col` collider switching & floor protection, `SaveGameData.interior` persistence, and interactive Johnson House (`CARLS` heal+save), Cluckin' Bell (`FDCHICK` `-$10, +50 HP`), and Ammu-Nation (`AMMUN*` `-$200, +50` 9mm reserve) counters.
- Real GTA:SA audio playback in `gtasarust-gta-formats` (`audio_cfg.rs`) and `gtasarust-audio` (`kira`): rotating 16-byte XOR Ogg Vorbis stream extraction (`StrmPaks.dat`/`TrakLkup.dat`, `offset + 8068`), uncompressed 16-bit PCM SFX bank reader (`PakFiles.dat`/`BankLkup.dat`, 4804-byte bank header), background radio decoding on `R`/vehicle entry (`264` tracks across `7` stations verified on `E:\gtasa`), `FEET`/`GENRL` footsteps, weapon shots, car horn, crash SFX, and procedural fallback SFX when running without `--game-dir`.
- `AGENTS.md` and `CONTINUE.md` autonomous resumption and verification handoff guides.
- `NODES*.DAT` road & pedestrian navigation graph parser (`crates/gta_formats/src/paths.rs`, verified across all 64 sectors: `68,237` nodes and `143,622` links) plus `PathGraph` spatial routing (`gtasarust-world`), sidewalk-patrolling pedestrians, ambient road traffic following vehicle physics/handling & yielding to obstacles, and carjacking crime/panic reactions (`gtasarust-ai`, `gtasarust-game`).
- Dynamic `.col` collision streaming (`10,160` models indexed from `models/coll/*.col`, `gta3.img`, `gta_int.img`), transformed `rapier3d` trimesh/box/sphere static colliders (`25,041` active around Grove Street), on-foot wall sliding, vehicle wall impact damage, and ground-raycast spawning on real Grove Street asphalt (`Z=12.712m`, `BUG-006`).
- Ambient parked vehicle spawning from binary IPL `CARS` sections (`1,043` entries across San Andreas) in `gtasarust-game`, plus `Car Spawns` reporting in `gtasarust-map-inspector` and binary IPL counting in `gtasarust-asset-pipeline`.
- Textured RenderWare DFF/TXD per-submesh rendering in `gtasarust-renderer` & `gtasarust-game` (`DffSubMesh`, hierarchical `DffFrame` transforms, vehicle primary/secondary paint tinting, `water.dat` 301-quad ocean/river mesh, and dynamic nearby DFF/TXD sector streaming; verified `68` models / `629` textures preloaded against `E:\gtasa`).
- Initial project governance, documentation, and issue/PR templates.
- 24-crate Rust 2024 workspace covering engine, `wgpu` rendering, `rapier3d` physics, `kira` audio, `winit`/`gilrs` input, world streaming, entities, vehicles, weapons, AI, missions, scripting, animation, `egui` UI, save system, asset pipeline, GTA formats, localization, `gtasarust` binary, and 4 CLI inspection/conversion tools (`asset-extractor`, `asset-converter`, `map-inspector`, `model-viewer`).
- Clean-room GTA:SA readers validated against PC DVD v1.00 (`E:\gtasa`): 8 IMG archives, 15,334 DFFs, 3,983 TXDs, 257 COLs, 14,344 IDE objects, 45,837 text+binary IPL placements, 16,588 GXT strings, 210 `handling.cfg` entries, `timecyc.dat`, `water.dat`, `weapon.dat`, `ped.ifp`, and `main.scm`.
- Cross-platform CI workflow for Windows x86_64 (`x86_64-pc-windows-msvc`) and Linux x86_64 (`x86_64-unknown-linux-gnu`).

### Fixed
- Fixed `IplSection::Enex` off-by-one field indexing (`fields[7..14]`) in `gtasarust-gta-formats/ipl.rs` so interior IDs, exit rotations, and doorway names parse cleanly (`CARLS` -> `interior=3`), and fixed `RW_EXTENSION` frame index advancing in `rw.rs` for `HAnim` bone attachment (`BUG-007`, `BUG-008`).
- Fixed binary `bnry` IPL `cars_offset` header read (`0x3C` instead of `0x40`) in `gtasarust-gta-formats` so all 1,043 binary `CARS` generators decode cleanly (`BUG-004`).
- Fixed alternating odd/even RenderWare `is_tristrip` winding order in `gtasarust-asset-pipeline` and `gtasarust-renderer`, applied hierarchical `DffFrame` world transforms cleanly, and guarded `upload_rgba8_texture` against empty/undersized TXD buffers (`BUG-005`).
- Enabled `egui/default_fonts` so HUD, main menu, pause menu, and debug inspector text renders properly at runtime.
- Handled 0-byte audio stream placeholders and `VER2` IMG streaming sector fallback safely.
