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
| Phase 2 | Engine | Tested | `winit` windowing (`Locked`/`Confined` pointer lock), `wgpu` 3D renderer (3 passes + mipmaps + sRGB gamma) + `egui` HUD/menus, `rapier3d` physics, `kira` audio, `gilrs`/KB input |
| Phase 3 | Original Asset Pipeline | Tested | Parsers for `VER2` IMG (8 archives), DFF (`0x34003`/`0x36003` + `SkinPLG`/`HAnim`/`ExtraVertColour`), TXD (`565`/`alpha_name`), COL1-4, IDE (`txdp`, flags), text & binary `bnry` IPL, GXT, DAT, `NODES*.DAT`, `audio/CONFIG`, `ANP3` IFP, `ScmImage` chunks + CLI tools |
| Phase 4 | World | Tested | `gta.dat`/`default.dat` + `gta_int.img` loader (14,344 IDE objs, 1,457 sectors, 50,935 placements, 1,045 `CARS`, 376 `ENEX`, 10,160 `.col`, 68,237 paths, `txdp` chains, time-of-day active checks, skyline LODs), 3-pass DFF/TXD rendering, `.col` physics streaming, water quads |
| Phase 5 | Player | Tested | Modular skinned CJ from `player.img` (`head`, `vest`, `jeans`, `sneaker`, `hands` + `ped.ifp`), decoupled orbit camera w/ `.col` anti-clip & mouse wheel zoom, camera-relative locomotion & aim turn, `.col` wall sliding, `ENEX` interiors |
| Phase 6 | Vehicles | In progress | Vehicle manager, `handling.cfg` (210 entries), `dff_vehicle_parts` 4-wheel mounts w/ steering & rolling spin, 4-point `.col` pitch/roll alignment, enter/exit switching, crash damage, IPL `CARS` & traffic |
| Phase 7 | World Simulation | In progress | `NODES*.DAT` sidewalk pedestrians (`fam2`/`fam3`/`bmydj`/`lapd1`/`hmost` skinned via `ped.ifp`) & ambient road traffic (yielding/stopping & carjacking panic), 6-star wanted escalation & cop spawning, `ENEX` shops/safehouse, 24h clock & timecycle (`dn_balance`, ambient obj, water color) |
| Phase 8 | Gameplay | In progress | Weapon inventory & fire/reload cooldowns, Johnson House safehouse heal+auto-save, Cluckin' Bell meal & Ammu-Nation ammo counters, save slot serialization w/ `interior` persistence & SHA-256 integrity checks |
| Phase 9 | Story | In progress | Retail `main.scm` chunk parser + `ScmVm` multi-thread bytecode VM (`699` SA opcodes, `ScmWorldCommand` dispatch) + checkpoint campaign triggers |
| Phase 10 | Final Game | In progress | Retail `LOADSCS.TXD` splash (`loadsc0`), `hud.txd` (`fist`, `radardisc`, radar blips), `radar00..radar143.txd` circular minimap & 12x12 fullscreen map, pause/settings/save menus, GXT localization |
| Phase 11 | Compatibility | Tested | Verified native builds & runtime checks on both Windows 10/11 (`x86_64-pc-windows-msvc`) and Ubuntu 22.04.5 LTS (`x86_64-unknown-linux-gnu` via WSL2) against `E:\gtasa` |
| Phase 12 | Release | Not started | Release packaging & final documentation |

## Subsystem Matrix

| Crate | Responsibility | Status | Evidence |
| --- | --- | --- | --- |
| `gtasarust-engine` | Platform paths, config, logging, lifecycle | Tested | 6 unit tests pass; VFS resolves `E:\gtasa` mixed-case paths |
| `gtasarust-gta-formats` | Clean-room SA binary/text parsers | Tested | 10 unit tests pass; inspected 15,334 DFF, 3,983 TXD (`alpha_name`, `565`), 257 COL, 68,237 path nodes, 16,588 GXT, `ANP3` `ped.ifp`, `ENEX`, audio indices, `ScmImage` chunks + 699 SA opcode shapes |
| `gtasarust-asset-pipeline` | Batch extraction, conversion, manifests | Tested | Unit test + `assets_suite` pass; converted DFF meshes to JSON/manifest cleanly |
| `gtasarust-renderer` | `wgpu` + WGSL 3D renderer & timecycle | Tested | 7 unit tests pass; Vulkan RX 480; 3-pass opaque/double-sided/blend pipelines, `Rgba8Unorm` + mipmaps, `ExtraVertColour`, `dff_vehicle_parts`, `.col` collided camera, preloaded 350 models / 3,008 textures |
| `gtasarust-physics` | `rapier3d` world/character/vehicle physics | Tested | 2 unit tests + integration suite pass; 25,041 active `.col` colliders grounded CJ at Grove Street (`Z=12.712m`); interior collider swapping & camera wall rays |
| `gtasarust-audio` | `kira` mixer, SFX, radio playback | Tested | Headless test + live Kira backend on Windows; decoded `BEATS` Ogg Vorbis (`6,597,202` frames @ `32kHz`), indexed 7 stations (`264` tracks) & loaded `10` SA PCM SFX (`FEET`/`GENRL`) |
| `gtasarust-input` | `winit` + `gilrs` input mapping | Tested | Compiles cleanly; `gilrs` initialized at runtime on Windows; pointer lock & scroll zoom wired |
| `gtasarust-world` | Map loading, sector streaming, zones, paths | Tested | 2 unit tests pass; loaded 50,935 placements (`1,457` sectors incl. `gta_int.img`), `txdp` TXD chains, `1,045` `CARS`, `376` `ENEX`, `10,160` `.col`, `68,237` paths |
| `gtasarust-entities` | Player, peds, pickups, world objects | Tested | Camera-relative locomotion, decoupled orbit yaw/pitch, aiming turn, armor damage, & `interior` state tested |
| `gtasarust-vehicles` | Vehicle physics, `handling.cfg`, seats/doors | Tested | Entry/exit, wheel spin/steering, & 4-point `.col` pitch/roll ground alignment tested |
| `gtasarust-weapons` | `weapon.dat`, combat, explosions | Tested | Handgun fire & reload test + gameplay integration test pass |
| `gtasarust-ai` | Ped, traffic, and police wanted AI | Tested | 2 unit tests pass; `NODES*.DAT` sidewalk pedestrians, traffic lane following/braking & carjack panic, wanted escalation |
| `gtasarust-animation` | IFP skeleton & animation state machine | Tested | 2 unit tests pass (`ped.ifp` 294 clips decoded; hierarchical `compute_skin_matrices` evaluated) |
| `gtasarust-scripting` | SCM bytecode VM & opcode dispatch | Tested | 2 unit tests pass; `ScmVm` loads `main.scm` (`main_offset=55976`, `135` missions) & dispatches `ScmWorldCommand` |
| `gtasarust-missions` | Campaign triggers, objectives, progression | Tested | Checkpoint progression & reward test passes |
| `gtasarust-localization` | GXT lookup & multilingual strings | Tested | Tag-stripping test passes; 16,588 `american.gxt` entries loaded |
| `gtasarust-ui` | SA HUD, radar, menus, `egui` dev tools | Tested | Retail `LOADSCS.TXD` splash, `hud.txd` (`69` sprites), `144` `radar00..radar143` circular minimap & 12x12 fullscreen map |
| `gtasarust-save-system` | Versioned save slots, checksums, migrations | Tested | Save roundtrip (with `interior` persistence) & corruption detection test passes |
| `gtasarust-game` | Playable `gtasarust` executable | Tested | 2 unit tests + smoke test (`main_offset=55976`, `skinned_fam1_moved_verts=1011`, `carls_paired_interior=Some(3)`) + 6s Vulkan RX 480 launch (`parts=5`, `models=350`, `vehicles=7`, `textures=3008`) pass |
| `gtasarust-tools` | CLI tools (`extract`, `convert`, `map`, `viewer`) | Tested | All 4 CLI binaries verified on `E:\gtasa` (`gta3.img`, `infernus.dff`, `hud.txd`) |

## Next Immediate Task
1. Expand `main.scm` mission/cutscene opcode dispatch for early Grove Street campaign progression (*In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood*).
2. Add specialized vehicle aerodynamics/buoyancy (motorcycles/BMX lean, boats, aircraft), aiming reticle & ped combat reactions, and Pay 'n' Spray / Mod Garages.
See `CONTINUE.md` for live resumption state.
