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
| Phase 3 | Original Asset Pipeline | Tested | Parsers for `VER2` IMG (8 archives + `script.img`/`cuts.img`), DFF (`0x34003`/`0x36003` + `SkinPLG`/`HAnim`/`ExtraVertColour`), TXD (`565`/`alpha_name`), COL1-4, IDE (`txdp`, flags), text & binary `bnry` IPL, GXT (`get_any`), `.cut`, DAT, `NODES*.DAT`, `audio/CONFIG`, `ANP3` IFP, `ScmImage` chunks + CLI tools |
| Phase 4 | World | Tested | `gta.dat`/`default.dat` + `gta_int.img` loader (14,344 IDE objs, 1,457 sectors, 50,935 placements, 1,045 `CARS`, 376 `ENEX`, 10,160 `.col`, 68,237 paths, `txdp` chains, time-of-day active checks, skyline LODs), 3-pass DFF/TXD rendering, `.col` physics streaming, `water.dat` 2D height + scrolling `waterclear256` |
| Phase 5 | Player | Tested | Modular skinned CJ from `player.img` (`head`, `vest`, `jeans`, `sneaker`, `hands` + `ped.ifp`), decoupled orbit + OTS aiming camera w/ `.col` anti-clip, right-hand weapon attachment (`BONE_R_HAND`), water swimming/breath/drowning, `ENEX` interiors |
| Phase 6 | Vehicles | In progress | Vehicle manager, dynamic kinematic box colliders (`VEHICLE_COLLIDER_TAG_FLAG`), `handling.cfg` (210 entries), `dff_vehicle_parts` 4-wheel mounts + helicopter main/tail rotor mounts, 4-point `.col` pitch/roll alignment, helicopter rotor spin & pitch/roll tilt, emergency red/blue roof siren flashers, boat buoyancy & deep-water stall, `< 250 HP` burn & explosion |
| Phase 7 | World Simulation | In progress | `NODES*.DAT` sidewalk pedestrians (`fam2`/`fam3`/`bmydj`/`lapd1`/`hmost` skinned via `ped.ifp`) & ambient road traffic (yielding/stopping & carjacking panic), 6-star wanted escalation & cop spawning, `ENEX` shops/safehouse, `grge` Pay 'n' Spray (`$100` repair/recolor/clear-wanted), 24h clock & timecycle, terrain ground shadows |
| Phase 8 | Gameplay | In progress | Weapon inventory + 9 preloaded weapon DFFs, 3D occluded combat raycasts & explosive blasts, underwater breath & drowning, Johnson House heal+save, Cluckin' Bell & Ammu-Nation counters, Pay 'n' Spray garages, save slot serialization w/ `interior` & SHA-256 checks |
| Phase 9 | Story | In progress | Retail `main.scm` (`135` missions) + `data/script/script.img` (`79` streamed scripts) + `anim/cuts.img` `.cut` cutscenes/subtitles in `ScmVm`, scripted cameras, player control locks, actor/car/checkpoint/weapon/armour/wanted/garage `ScmWorldCommand` dispatch + checkpoint campaign triggers |
| Phase 10 | Final Game | In progress | Retail `LOADSCS.TXD` splash (`loadsc0`), `hud.txd` (`69` sprites incl. `siteM16` reticle, `radar_spray` & weapon icons), `radar00..143` minimap & fullscreen map, cutscene widescreen letterbox + rich GXT subtitles, `CHudColours` spans, fullscreen `sa_screen_fade`, RPM looping engine audio + `Respray`/`Siren` SFX, pause/settings/save menus |
| Phase 11 | Compatibility | Tested | Verified `fmt`, `clippy -D warnings`, `62/62` workspace tests, and release `--smoke-test` (`60` frames) on both Windows 10/11 (`x86_64-pc-windows-msvc`) and Ubuntu 22.04 (`x86_64-unknown-linux-gnu` via WSL2) against `E:\gtasa`, plus live 5s Vulkan RX 480 launch |
| Phase 12 | Release | Not started | Release packaging & final documentation |

## Subsystem Matrix

| Crate | Responsibility | Status | Evidence |
| --- | --- | --- | --- |
| `gtasarust-engine` | Platform paths, config, logging, lifecycle | Tested | 6 unit tests pass; VFS resolves `E:\gtasa` mixed-case paths |
| `gtasarust-gta-formats` | Clean-room SA binary/text parsers | Tested | 11 unit tests pass; inspected IMG/`script.img`/`cuts.img`, DFF, TXD, COL1-4, paths, `american.gxt` (`get_any`), `.cut`, `ANP3` `ped.ifp`, `ENEX`, audio indices, `ScmImage` chunks |
| `gtasarust-asset-pipeline` | Batch extraction, conversion, manifests | Tested | Unit test + `assets_suite` pass; converted DFF meshes to JSON/manifest cleanly |
| `gtasarust-renderer` | `wgpu` + WGSL 3D renderer & timecycle | Tested | 9 unit tests pass; Vulkan RX 480; 3-pass pipelines, OTS camera + `aim_ray`, `unit_shadow_quad`, animated `waterclear256` scrolling shader, `ExtraVertColour`, `dff_vehicle_parts` (wheels + `VehicleRotorMount`), preloaded 359 models / 3,054 textures |
| `gtasarust-physics` | `rapier3d` world/character/vehicle physics | Tested | 3 unit tests + integration suite pass; 25,049 colliders (`25,041` static `.col` + 8 dynamic vehicle boxes), kinematic pose updates & `raycast_excluding` |
| `gtasarust-audio` | `kira` mixer, SFX, radio playback | Tested | Headless test + live Kira backend on Windows; decoded `BEATS` Ogg Vorbis (`6,597,202` frames @ `32kHz`), indexed 7 stations (`264` tracks), `11` SA PCM SFX, RPM-scaled looping engine audio + `Respray`/`Siren` SFX |
| `gtasarust-input` | `winit` + `gilrs` input mapping | Tested | Compiles cleanly; `gilrs` initialized at runtime on Windows; pointer lock, scroll zoom & `skip_cutscene_just_pressed` (`Space`/`Enter`) wired |
| `gtasarust-world` | Map loading, sector streaming, zones, paths | Tested | 3 unit tests pass; 50,935 placements (`1,457` sectors), `txdp` chains, `water_height_at` barycentric lookup, `1,045` `CARS`, `376` `ENEX`, `10,160` `.col`, `68,237` paths |
| `gtasarust-entities` | Player, peds, pickups, world objects | Tested | 2 unit tests pass; camera-relative movement, decoupled orbit yaw/pitch, aiming turn, water swimming/breath/drowning, armor damage & `interior` state |
| `gtasarust-vehicles` | Vehicle physics, `handling.cfg`, seats/doors | Tested | 2 unit tests pass; `step_physics_with_context` self-tag exclusion, entry/exit, 4-wheel spin/steering & `.col` alignment, boat buoyancy/stall, `< 250 HP` burn & explosion |
| `gtasarust-weapons` | `weapon.dat`, combat, explosions | Tested | Handgun fire & reload test + gameplay & collider/explosion tests pass |
| `gtasarust-ai` | Ped, traffic, and police wanted AI | Tested | 2 unit tests pass; `NODES*.DAT` sidewalk pedestrians, traffic lane following/braking & carjack panic, wanted escalation |
| `gtasarust-animation` | IFP skeleton & animation state machine | Tested | 2 unit tests pass (`ped.ifp` 294 clips, `BONE_R_HAND = 24`, `compute_bone_world_matrix` & `compute_skin_matrices`) |
| `gtasarust-scripting` | SCM bytecode VM & opcode dispatch | Tested | 4 unit tests pass; `ScmVm` loads `main.scm` (`main_offset=55976`, `135` missions) + `script.img` (`79` scripts) & dispatches camera/cutscene/control/weapon/armour/wanted/garage `ScmWorldCommand` |
| `gtasarust-missions` | Campaign triggers, objectives, progression | Tested | Checkpoint progression & reward test passes |
| `gtasarust-localization` | GXT lookup & multilingual strings | Tested | 2 unit tests pass; `american.gxt` (`lookup_any`) + `CHudColours` `parse_sa_rich_text` (`~r~/~g~/~b~/~y~/~p~/~h~/~n~`) |
| `gtasarust-ui` | SA HUD, radar, menus, `egui` dev tools | Tested | Retail `LOADSCS.TXD` splash, `hud.txd` (`69` sprites incl. `siteM16`, `radar_spray` & weapons), `144` `radar00..143` minimap & map, breath bar, cutscene letterbox & subtitles, fade curtain |
| `gtasarust-save-system` | Versioned save slots, checksums, migrations | Tested | Save roundtrip (with `interior` persistence) & corruption detection test passes |
| `gtasarust-game` | Playable `gtasarust` executable | Tested | 7 unit tests + smoke test (`main_offset=55976`, `streamed_scripts=79`, `skinned_fam1_moved_verts=1011`) + 5s Vulkan RX 480 launch (`colliders=25049`, `models=359`, `vehicles=8`, `textures=3054`) pass |
| `gtasarust-tools` | CLI tools (`extract`, `convert`, `map`, `viewer`) | Tested | All 4 CLI binaries verified on `E:\gtasa` (`gta3.img`, `infernus.dff`, `hud.txd`) |

## Next Immediate Task
1. Milestone H1: Expand `main.scm` mission progression & opcode coverage (story mission scripts, scripted ped/car tasks, gang recruitment, phone calls, blips/coronas) through early Los Santos missions (*In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood*).
2. Milestone H1: Add motorcycle/BMX lean & wheelies, fixed-wing plane aerodynamics/landing gear, Mod Garages / gang territory wars, and ambient ped speech/combat AI.
See `CONTINUE.md` for live resumption state.
