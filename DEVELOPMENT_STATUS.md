# GTASARUST Development Status

Status Legend:
- **Not started**: Design planned, no implementation yet.
- **In progress**: Under active implementation.
- **Implemented**: Core logic compiles and integrates with workspace.
- **Tested**: Automated unit/integration tests pass.
- **Verified**: Verified on both Windows x86_64 and Linux x86_64 (and against `E:\gtasa` where applicable).

## Phase Tracker

| Phase | Name | Status | Notes |
| --- | --- | --- | --- |
| Phase 1 | Foundation | Tested | 24-crate Cargo workspace, `x86_64`-only enforcement, TOML config, tracing logging, case-insensitive VFS, CI |
| Phase 2 | Engine | Tested | `winit` windowing, `wgpu` 3D renderer + frustum culling + `egui` HUD/menus, `rapier3d` physics, `kira` audio, `gilrs`/KB input |
| Phase 3 | Original Asset Pipeline | Tested | Parsers for `VER2` IMG (8 archives), DFF (`0x34003`/`0x36003`), TXD, COL1-4, IDE, text & binary `bnry` IPL, `TABL`/`TKEY`/`TDAT` GXT, DAT, IFP, SCM header + CLI tools |
| Phase 4 | World | Tested | `gta.dat`/`default.dat` loader (14,344 IDE objs, 1,455 sectors, 45,837 placements, 1,043 `CARS`, 10,160 `.col` models, 68,237 `NODES*.DAT` waypoints, 384 zones, 301 `water.dat` quads), textured DFF/TXD per-submesh rendering, `.col` physics streaming |
| Phase 5 | Player | In progress | On-foot controller (walk/sprint/jump/crouch/health/armor), `.col` wall sliding & ground elevation (`Z=12.71m`), hospital respawn, third-person camera |
| Phase 6 | Vehicles | In progress | Vehicle manager, `handling.cfg` (210 entries), enter/exit switching, `.col` wall collision damage, IPL `CARS` parked spawns & `NODES*.DAT` ambient traffic |
| Phase 7 | World Simulation | In progress | `NODES*.DAT` sidewalk pedestrians & ambient road traffic (yielding/stopping & carjacking panic), 6-star wanted escalation & cop spawning, 24h clock & timecycle |
| Phase 8 | Gameplay | In progress | Weapon inventory & fire/reload cooldowns, save slot serialization w/ SHA-256 integrity checks |
| Phase 9 | Story | In progress | Campaign mission framework & early SCM VM scaffolding (bytecode execution expansion pending) |
| Phase 10 | Final Game | In progress | Main menu, pause menu, SA HUD/minimap stub, settings menu, localization w/ GXT + fallback JSON |
| Phase 11 | Compatibility | Tested | Verified native builds & runtime checks on both Windows 10/11 (`x86_64-pc-windows-msvc`) and Ubuntu 22.04.5 LTS (`x86_64-unknown-linux-gnu` via WSL2) against `E:\gtasa` |
| Phase 12 | Release | Not started | Release packaging & final documentation |

## Subsystem Matrix

| Crate | Responsibility | Status | Evidence |
| --- | --- | --- | --- |
| `gtasarust-engine` | Platform paths, config, logging, lifecycle | Tested | 6 unit tests pass; VFS resolves `E:\gtasa` mixed-case paths |
| `gtasarust-gta-formats` | Clean-room SA binary/text parsers | Tested | 6 unit tests pass; inspected 15,334 DFF, 3,983 TXD, 257 COL, 68,237 path nodes, 16,588 GXT on `E:\gtasa` |
| `gtasarust-asset-pipeline` | Batch extraction, conversion, manifests | Tested | Unit test + `assets_suite` pass; converted DFF meshes to JSON/manifest cleanly |
| `gtasarust-renderer` | `wgpu` + WGSL 3D renderer & timecycle | Tested | 3 unit tests pass; initialized Vulkan surface on Radeon RX 480; preloaded 68 DFF models / 629 TXD textures + `water.dat` mesh |
| `gtasarust-physics` | `rapier3d` world/character/vehicle physics | Tested | 2 unit tests + integration suite pass; 25,041 active `.col` colliders grounded CJ at Grove Street (`Z=12.712m`) |
| `gtasarust-audio` | `kira` mixer, SFX, radio playback | Tested | Headless radio test passes; live `Kira` audio backend initialized on Windows |
| `gtasarust-input` | `winit` + `gilrs` input mapping | Tested | Compiles cleanly; `gilrs` initialized at runtime on Windows |
| `gtasarust-world` | Map loading, sector streaming, zones, paths | Tested | 2 unit tests pass; loaded 45,837 placements, 1,043 `CARS`, 10,160 `.col` models, 68,237 `NODES*.DAT` waypoints |
| `gtasarust-entities` | Player, peds, pickups, world objects | Tested | Player movement & armor damage test passes |
| `gtasarust-vehicles` | Vehicle physics, `handling.cfg`, seats/doors | Tested | Entry/exit & driving physics test passes; 210 `handling.cfg` entries loaded |
| `gtasarust-weapons` | `weapon.dat`, combat, explosions | Tested | Handgun fire & reload test + gameplay integration test pass |
| `gtasarust-ai` | Ped, traffic, and police wanted AI | Tested | 2 unit tests pass; `NODES*.DAT` sidewalk pedestrians, traffic lane following/braking & carjack panic, wanted escalation |
| `gtasarust-animation` | IFP skeleton & animation state machine | Tested | Blend transition test passes (`ped.ifp` 294 clips indexed) |
| `gtasarust-scripting` | SCM bytecode VM & opcode dispatch | Tested | VM wait/effect test passes (`main.scm` 135 missions scanned) |
| `gtasarust-missions` | Campaign triggers, objectives, progression | Tested | Checkpoint progression & reward test passes |
| `gtasarust-localization` | GXT lookup & multilingual strings | Tested | Tag-stripping test passes; 16,588 `american.gxt` entries loaded |
| `gtasarust-ui` | SA HUD, radar, menus, `egui` dev tools | Tested | Compiles clippy-clean; `egui` `default_fonts` enabled and verified |
| `gtasarust-save-system` | Versioned save slots, checksums, migrations | Tested | Save roundtrip & corruption detection test passes |
| `gtasarust-game` | Playable `gtasarust` executable | Tested | 60-tick headless smoke test + 4s windowed GPU startup pass against `E:\gtasa` |
| `gtasarust-tools` | CLI tools (`extract`, `convert`, `map`, `viewer`) | Tested | All 4 CLI binaries verified on `E:\gtasa` (`gta3.img`, `infernus.dff`, `hud.txd`) |

## Next Immediate Task
1. Decode `audio/streams` Ogg Vorbis radio tracks & SFX banks into `gtasarust-audio` (`kira`) real playback.
2. Expand `ped.ifp` skeletal bone keyframes (`gtasarust-animation`) and `main.scm` bytecode opcodes (`gtasarust-scripting`).
3. Render `ENEX` interior markers & teleportation transitions plus shops/safehouse saves.
