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
| Phase 3 | Original Asset Pipeline | Tested | Parsers for `VER2` IMG (8 archives), DFF (`0x34003`/`0x36003` + `SkinPLG`/`HAnim`), TXD, COL1-4, IDE, text & binary `bnry` IPL (`ENEX`, `CARS`, `gta_int.img`), `TABL`/`TKEY`/`TDAT` GXT, DAT, `NODES*.DAT`, `audio/CONFIG`, `ANP3` IFP keyframes (`ped.ifp`), SCM header + CLI tools |
| Phase 4 | World | Tested | `gta.dat`/`default.dat` + `gta_int.img` loader (14,344 IDE objs, 1,457 sectors, 50,935 placements, 1,045 `CARS`, 376 `ENEX` portals, 10,160 `.col` models, 68,237 `NODES*.DAT` waypoints, 384 zones, 301 `water.dat` quads), textured DFF/TXD per-submesh rendering, `.col` physics streaming, `ENEX` interior transitions |
| Phase 5 | Player | Tested | On-foot controller (walk/sprint/jump/crouch/health/armor), `ped.ifp` CPU-skinned `fam1` CJ rendering (`1,011` verts deformed), `.col` wall sliding & elevation (`Z=12.71m`), `ENEX` interior transitions & floor safety, hospital respawn, third-person camera |
| Phase 6 | Vehicles | In progress | Vehicle manager, `handling.cfg` (210 entries), enter/exit switching, `.col` wall collision damage, IPL `CARS` parked spawns & `NODES*.DAT` ambient traffic |
| Phase 7 | World Simulation | In progress | `NODES*.DAT` sidewalk pedestrians (`fam2`/`fam3`/`bmydj`/`lapd1`/`hmost` skinned via `ped.ifp`) & ambient road traffic (yielding/stopping & carjacking panic), 6-star wanted escalation & cop spawning, `ENEX` shops/safehouse, 24h clock & timecycle |
| Phase 8 | Gameplay | In progress | Weapon inventory & fire/reload cooldowns, Johnson House safehouse heal+auto-save, Cluckin' Bell meal & Ammu-Nation ammo counters, save slot serialization w/ `interior` persistence & SHA-256 integrity checks |
| Phase 9 | Story | In progress | Campaign mission framework & early SCM VM scaffolding (bytecode execution expansion pending) |
| Phase 10 | Final Game | In progress | Main menu, pause menu, SA HUD/radar w/ `ENEX` doorway blips & top-left help toast, settings menu, localization w/ GXT + fallback JSON |
| Phase 11 | Compatibility | Tested | Verified native builds & runtime checks on both Windows 10/11 (`x86_64-pc-windows-msvc`) and Ubuntu 22.04.5 LTS (`x86_64-unknown-linux-gnu` via WSL2) against `E:\gtasa` |
| Phase 12 | Release | Not started | Release packaging & final documentation |

## Subsystem Matrix

| Crate | Responsibility | Status | Evidence |
| --- | --- | --- | --- |
| `gtasarust-engine` | Platform paths, config, logging, lifecycle | Tested | 6 unit tests pass; VFS resolves `E:\gtasa` mixed-case paths |
| `gtasarust-gta-formats` | Clean-room SA binary/text parsers | Tested | 9 unit tests pass; inspected 15,334 DFF (`SkinPLG`/`HAnim`), 3,983 TXD, 257 COL, 68,237 path nodes, 16,588 GXT, `ANP3` `ped.ifp`, `ENEX`, `StrmPaks`/`TrakLkup` & `PakFiles`/`BankLkup` on `E:\gtasa` |
| `gtasarust-asset-pipeline` | Batch extraction, conversion, manifests | Tested | Unit test + `assets_suite` pass; converted DFF meshes to JSON/manifest cleanly |
| `gtasarust-renderer` | `wgpu` + WGSL 3D renderer & timecycle | Tested | 5 unit tests pass; Vulkan surface on Radeon RX 480; preloaded 71 DFF models / 656 TXD textures + dynamic skinned ped buffers & `marker_cone` + `water.dat` |
| `gtasarust-physics` | `rapier3d` world/character/vehicle physics | Tested | 2 unit tests + integration suite pass; 25,041 active `.col` colliders grounded CJ at Grove Street (`Z=12.712m`); interior collider swapping |
| `gtasarust-audio` | `kira` mixer, SFX, radio playback | Tested | Headless test + live Kira backend on Windows; decoded `BEATS` Ogg Vorbis (`6,597,202` frames @ `32kHz`), indexed 7 stations (`264` tracks) & loaded `10` SA PCM SFX (`FEET`/`GENRL`) |
| `gtasarust-input` | `winit` + `gilrs` input mapping | Tested | Compiles cleanly; `gilrs` initialized at runtime on Windows |
| `gtasarust-world` | Map loading, sector streaming, zones, paths | Tested | 2 unit tests pass; loaded 50,935 placements (`1,457` sectors incl. `gta_int.img`), `1,045` `CARS`, `376` `ENEX` portals, 10,160 `.col` models, 68,237 `NODES*.DAT` waypoints |
| `gtasarust-entities` | Player, peds, pickups, world objects | Tested | Player movement, armor damage, & `interior` state tested |
| `gtasarust-vehicles` | Vehicle physics, `handling.cfg`, seats/doors | Tested | Entry/exit & driving physics test passes; 210 `handling.cfg` entries loaded |
| `gtasarust-weapons` | `weapon.dat`, combat, explosions | Tested | Handgun fire & reload test + gameplay integration test pass |
| `gtasarust-ai` | Ped, traffic, and police wanted AI | Tested | 2 unit tests pass; `NODES*.DAT` sidewalk pedestrians, traffic lane following/braking & carjack panic, wanted escalation |
| `gtasarust-animation` | IFP skeleton & animation state machine | Tested | 2 unit tests pass (`ped.ifp` 294 clips decoded; hierarchical `compute_skin_matrices` evaluated) |
| `gtasarust-scripting` | SCM bytecode VM & opcode dispatch | Tested | VM wait/effect test passes (`main.scm` 135 missions scanned) |
| `gtasarust-missions` | Campaign triggers, objectives, progression | Tested | Checkpoint progression & reward test passes |
| `gtasarust-localization` | GXT lookup & multilingual strings | Tested | Tag-stripping test passes; 16,588 `american.gxt` entries loaded |
| `gtasarust-ui` | SA HUD, radar, menus, `egui` dev tools | Tested | HUD help toast, radar `ENEX` doorway squares, and interior dev readout verified |
| `gtasarust-save-system` | Versioned save slots, checksums, migrations | Tested | Save roundtrip (with `interior` persistence) & corruption detection test passes |
| `gtasarust-game` | Playable `gtasarust` executable | Tested | `enex_teleports_to_interior_and_triggers_services` test + 60-tick smoke test (`ifp_clips=294`, `skinned_fam1_moved_verts=1011`, `carls_paired_interior=Some(3)`) + 4s Vulkan launch pass |
| `gtasarust-tools` | CLI tools (`extract`, `convert`, `map`, `viewer`) | Tested | All 4 CLI binaries verified on `E:\gtasa` (`gta3.img`, `infernus.dff`, `hud.txd`) |

## Next Immediate Task
1. Expand `main.scm` bytecode opcodes & multi-thread scheduler (`gtasarust-scripting`).
2. Wire early Grove Street story missions (*In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood*) into live gameplay (`gtasarust-missions`, `gtasarust-game`).
3. Expand specialized vehicle classes, water buoyancy, gang zones, and Mod/Pay 'n' Spray garages.
See `CONTINUE.md` for live resumption state.
