# CONTINUE.md — GTASARUST Live Handoff & Resume State

> **Notice to any resuming AI agent:** Read `AGENTS.md` and this file first, then execute the **Current Next Task** directly without re-auditing completed phases unless tests fail. Update this file and commit + push to `main` after every verified milestone.

---

## 1. Current Verified State (Milestone F1+F2 Baseline)

- **Repository**: `E:\prjects\GTASARUST` (`https://github.com/pon4ikisdonut/GTASARUST.git`, branch `main`)
- **Last Completed Milestone**: **Milestone F1+F2 — Authentic SA Rendering/Lighting, Modular `player.img` CJ, 4-Wheel Vehicle Rigs, Real `hud.txd`/`LOADSCS.TXD`/`radar00..143` UI & `main.scm` Chunk/Bytecode VM**
- **Previous Verified Commits on `main`**:
  - `dd47d1d` — Initial verified 24-crate Rust workspace on Windows + Linux x86_64
  - `7d5e8c9` — **Milestone A**: Textured RenderWare DFF/TXD per-submesh rendering, `DffFrame` hierarchy, alternating tristrip winding parity, vehicle paint tinting, `water.dat` 301-quad mesh, sector asset streaming (`BUG-005` fixed)
  - `5e19b6e` — **Milestone B**: `.col` collision streaming (`10,160` models / `25,041` active `rapier3d` colliders at Grove Street), on-foot wall sliding & spawn grounding (`Z=12.712m`), vehicle crash damage, binary `bnry` IPL `cars_offset` fix at `0x3C` (`1,043` `CARS`), parked traffic (`BUG-004`, `BUG-006` fixed)
  - `3f5c0f5` — **Milestone C**: All 64 `NODES*.DAT` road/ped path graphs (`68,237` nodes / `143,622` links) driving sidewalk pedestrians, moving ambient road vehicles w/ obstacle yielding, and carjacking crime/panic
  - `3f4a1e0` — **Milestone D**: Real `audio/streams` XOR-unscrambled Ogg Vorbis radio (`264` tracks across `7` stations) & `audio/SFX` 16-bit PCM bank playback (`10` clips from `FEET`/`GENRL`) via `kira 0.12.6`, plus `AGENTS.md` / `CONTINUE.md`
  - `282c342` — **Milestone E**: Full `ANP3` `ped.ifp` decoder (`294` clips), `SkinPLG`/`HAnim` CPU skinning (`fam1`, `1,011` verts deformed), `gta_int.img` binary IPLs (`50,935` placements / `376` `ENEX` portals), safehouse & Cluckin' Bell / Ammu-Nation counters (`BUG-007`, `BUG-008` fixed)
  - **Milestone F1+F2 (current commit)**:
    - **Visuals, Lighting & World (`assets/shaders/{world,sky}.wgsl`, `crates/{gta_formats,world,renderer}`)**: Fixed double-sRGB darkening (`Rgba8Unorm` texture upload + full 2x2 mipmap pyramid + sRGB gamma output on `Bgra8Unorm`), `ExtraVertColour` (`0x0253F2F9`) night prelit buffer + `timecyc.dat` ambient/ambient_obj/water/`dn_balance` interpolation, `DXT1` `FORMAT_565` opaque decoding + 8-bit luminance + `merge_rgba_alpha_mask`, `txdp` parent TXD chains across IDEs/`txd.ide`, 3-pass rendering (`world_pipeline`, `world_double_sided_pipeline`, `world_blend_pipeline`), `IDE_FLAG_*` cutout/glass/backface routing, `is_active_at_hour` building schedules, skyline LOD switching outside detail cap, `bounding_radius` frustum culling, omitted fallback ground quad when real map loaded, `water_mesh` rendered in transparent unlit pass
    - **Player, Camera & Vehicles (`crates/{entities,vehicles,renderer,game}`)**: Decoupled third-person camera orbit (`camera_yaw`/`camera_pitch`/`camera_distance` + mouse wheel zoom + `CursorGrabMode::Locked`/`Confined` pointer lock), `.col` raycast camera anti-clipping (`update_third_person_collided`), camera-relative locomotion & aiming yaw, modular skinned CJ loaded directly from `models/player.img` (`head`, `vest`, `jeans`, `sneaker`, `hands` animated by `ped.ifp`), `CpuMeshData::dff_vehicle_parts` separating vehicle body + `wheel` + 4 `wheel_*_dummy` mounts w/ steering, rolling, left/right flip, and 4-point `.col` pitch/roll alignment
    - **Retail HUD/Radar & `main.scm` VM (`crates/{gta_formats,scripting,ui,game}`)**: `LOADSCS.TXD` (`loadsc0`) splash screen + `"RESUME FREE ROAM"`, `models/hud.txd` (`69` sprites incl. `fist`, `radardisc`, `radar_north`, `radar_centre`, `radar_sweet`, `radar_cj`), `144` retail `radar00..radar143.txd` minimap & fullscreen textured `Map` view; full `ScmImage` chunk parser (`'s'`, `0..=4`) + 699 SA opcode descriptors (`scm_opcodes.rs`) derived from `D:\gta-reversed` & `ScmVm` multi-thread bytecode execution emitting `ScmWorldCommand` (`SetTimeOfDay`, `ForceWeather`, `CreateCar`, `CreatePickup`, `AddBlip`, `PrintText`, etc.)
    - Verified across **Windows 10/11 x86_64** (`46` workspace tests pass, `clippy -D warnings`, `--smoke-test`, 6s live Vulkan RX 480 launch: `main_offset=55976`, `hud_sprites=69`, `radar_tiles=144`, `parts=5` CJ, `models=350`, `vehicles=7`, `textures=3008`) and **WSL2 Ubuntu x86_64** (`fmt`, `clippy`, `46` workspace tests pass, release build, headless smoke, `gtasarust-map-inspector`)

- **Open Blockers**: None (`BUG-001`..`BUG-012` resolved in `BUGS.md`).

---

## 2. Current Next Task — Milestone G: Story Campaign Execution, Cutscenes & Full Gameplay Systems

1. **Extend `main.scm` Opcode Dispatch & Cutscene / Mission Scripting (`crates/scripting`, `crates/missions`, `crates/game`)**:
   - Wire additional high-frequency SA script opcodes (actors, vehicles, mission cleanup, garage/shop scripts, camera fades/scripted cams, `LOAD_AND_LAUNCH_MISSION_INTERNAL` streaming into *In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood* end-to-end).
   - Add cutscene `.cut`/text playback and synchronized scripted actor paths.
2. **Deepen Gameplay Systems (`crates/vehicles`, `crates/ai`, `crates/weapons`, `crates/audio`)**:
   - Motorbike/BMX lean, boat water buoyancy on `water.dat`, helicopter/plane flight lift & rotor spin, weapon aiming reticle (`siteM16`) & ped hit reactions, Pay 'n' Spray / Mod Garages, gang territory wars, and engine/siren audio synthesis.
3. **Verify & Ship Milestone G**:
   - Maintain full Windows x86_64 + WSL2 Linux x86_64 parity (`cargo fmt`, `clippy -- -D warnings`, `cargo test`, `--smoke-test`, live Vulkan launch), keep documentation synchronized, and push regularly to `main`.

---

## 3. Subsequent Roadmap (After Milestone G)

- **Milestone H — Full Story Arc, Audio Streams & Polish**:
  - Complete Los Santos -> San Fierro -> Las Venturas mission chains, radio DJ/commercial banter, ped speech banks, full indoor activities, and PC retail config/performance polish.

---

## 4. Quick Environment Notes for Resuming Agents

- **Game Installation Path**:
  - Windows: `E:\gtasa`
  - WSL2 Linux: `/mnt/e/gtasa`
- **Cargo Build Cache Note**: `E:` drive has ~14 GB free while `D:\gtasarust-target` holds Windows debug/release artifacts (144 GB free on `D:`); prefer `$env:CARGO_TARGET_DIR='D:\gtasarust-target'` on Windows and keep `E:\prjects\GTASARUST\target` clean so `E:` stays free for source/Git.
- **GitHub Action Runner Note**: Remote CI workflows may fail immediately with account runner billing disabled; always verify Linux `x86_64-unknown-linux-gnu` locally inside WSL2 using `CARGO_TARGET_DIR="$HOME/gtasarust-linux-target"` before pushing.
- **Last Windows & Linux Smoke Benchmark Output** (for regression comparison):
  - Map Inspector: `Objects: 14344, Vehicles: 198, Peds: 275, Sectors: 1457, Total Placements: 50935, Car Spawns: 1045, Path Nodes: 68237, Zones: 384, Water Quads: 301`
  - Smoke `--game-dir E:\gtasa`: `ticks=60 player_pos=Vec3(2503.5032, -1688.2, 0.0) zone=LA01 vehicles=3 handling_entries=210 audio_stations=7 audio_sfx_clips=10 ifp_clips=294 enex_portals=376 skinned_fam1_moved_verts=1011 carls_paired_interior=Some(3) decoded_radio=Some(("BEATS", 6597202, 32000))`
  - GPU Window Launch: `player_pos=Vec3(2495.3, -1688.2, 12.712079) colliders=25041 vehicles=8 models=71 textures=656` (`Vulkan Radeon RX 480`, `Kira audio backend initialized`).
