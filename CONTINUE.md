# CONTINUE.md — GTASARUST Live Handoff & Resume State

> **Notice to any resuming AI agent:** Read `AGENTS.md` and this file first, then execute the **Current Next Task** directly without re-auditing completed phases unless tests fail. Update this file and commit + push to `main` after every verified milestone.

---

## 1. Current Verified State (Milestone G1 Baseline)

- **Repository**: `E:\prjects\GTASARUST` (`https://github.com/pon4ikisdonut/GTASARUST.git`, branch `main`)
- **Last Completed Milestone**: **Milestone G1 — Water Swimming/Drowning, Vehicle Kinematic Colliders & Fire/Explosions, OTS Aiming & Wall-Occluded Shooting, Ground Shadows & Rich GXT/Fade HUD**
- **Previous Verified Commits on `main`**:
  - `dd47d1d` — Initial verified 24-crate Rust workspace on Windows + Linux x86_64
  - `7d5e8c9` — **Milestone A**: Textured RenderWare DFF/TXD per-submesh rendering, `DffFrame` hierarchy, alternating tristrip winding parity, vehicle paint tinting, `water.dat` 301-quad mesh, sector asset streaming (`BUG-005` fixed)
  - `5e19b6e` — **Milestone B**: `.col` collision streaming (`10,160` models / `25,041` active `rapier3d` colliders at Grove Street), on-foot wall sliding & spawn grounding (`Z=12.712m`), vehicle crash damage, binary `bnry` IPL `cars_offset` fix at `0x3C` (`1,043` `CARS`), parked traffic (`BUG-004`, `BUG-006` fixed)
  - `3f5c0f5` — **Milestone C**: All 64 `NODES*.DAT` road/ped path graphs (`68,237` nodes / `143,622` links) driving sidewalk pedestrians, moving ambient road vehicles w/ obstacle yielding, and carjacking crime/panic
  - `3f4a1e0` — **Milestone D**: Real `audio/streams` XOR-unscrambled Ogg Vorbis radio (`264` tracks across `7` stations) & `audio/SFX` 16-bit PCM bank playback (`10` clips from `FEET`/`GENRL`) via `kira 0.12.6`, plus `AGENTS.md` / `CONTINUE.md`
  - `282c342` — **Milestone E**: Full `ANP3` `ped.ifp` decoder (`294` clips), `SkinPLG`/`HAnim` CPU skinning (`fam1`, `1,011` verts deformed), `gta_int.img` binary IPLs (`50,935` placements / `376` `ENEX` portals), safehouse & Cluckin' Bell / Ammu-Nation counters (`BUG-007`, `BUG-008` fixed)
  - `06e45a1` — **Milestone F1+F2**: Fixed double-sRGB darkening & DXT1 565 alpha/txdp chains/3-pass `IDE_FLAG_*` rendering, `ExtraVertColour` day/night prelit buffers + `timecyc.dat` interpolation, modular `player.img` CJ, decoupled orbit camera w/ `.col` anti-clipping, `dff_vehicle_parts` 4-wheel mounts, retail `LOADSCS.TXD`/`hud.txd`/`radar00..143.txd` UI, `ScmImage` chunk parser + `ScmVm` (`699` SA opcodes)
  - **Milestone G1 (current commit)**:
    - **Water, Swimming & Vehicle Physics (`crates/{world,entities,vehicles,physics,game}`)**: Exact `water.dat` 2D triangle barycentric height (`WorldManager::water_height_at` + ocean bounds), CJ swimming buoyancy, breath depletion (`breath`), drowning damage & underwater blue HUD bar; dynamic per-vehicle `rapier3d` kinematic box colliders (`VEHICLE_COLLIDER_TAG_FLAG`) w/ self-tag exclusion (`raycast_excluding`), boat buoyancy & deep-water engine stall, `< 250 HP` burning countdown -> single explosion detonation (`SfxCue::Explosion` + `GENRL` 45/0 + charred wreck + blast flash)
    - **Combat, OTS Camera & Held Weapons (`crates/{renderer,animation,ui,game,audio}`)**: Over-the-shoulder aiming camera (`CameraState::update_over_shoulder` + `aim_ray`) w/ narrowed aiming FOV, `siteM16` reticle & `hud.txd` active weapon icon (`weapon_hud_sprite_name`), preloaded 9 handheld weapon DFFs (`colt45`..`knifecur`) mounted to CJ's right hand bone (`BONE_R_HAND = 24` via `compute_bone_world_matrix`), 3D wall/vehicle/ped-occluded shooting raycasts & explosive blasts
    - **Shadows, Water Shader & GXT/Fade UI (`assets/shaders/world.wgsl`, `crates/{localization,renderer,ui,game}`)**: `particle.txd` `shad_*`/`bloodpool_64` luminance-to-alpha conversion (`convert_shadow_luminance_to_alpha`), projected `.col` terrain shadows (`shad_ped`/`shad_car`), scrolling animated `waterclear256` shader (`light_params.x > 2.5`), `CHudColours` `~r~/~g~/~b~/~y~/~p~/~w~/~s~/~l~/~h~/~n~` rich-text GXT `LayoutJob` rendering (`parse_sa_rich_text`), and SCM `DoFade`/`DisplayHud`/`DisplayRadar` fullscreen curtain & HUD overrides
    - Verified across **Windows 10/11 x86_64** (`54/54` workspace tests pass, `clippy -D warnings`, `fmt --check`, `--smoke-test`, 5s live Vulkan RX 480 launch: `colliders=25049 vehicles=8 models=359 textures=3054 backdrop=true hud_sprites=69 radar_tiles=144 sa_sfx_loaded=11`) and **WSL2 Ubuntu x86_64** (`fmt --check`, `clippy -D warnings`, `54/54` workspace tests pass, release `--smoke-test`, `gtasarust-map-inspector`)

- **Open Blockers**: None (`BUG-001`..`BUG-014` resolved in `BUGS.md`).

---

## 2. Current Next Task — Milestone G2: Retail `main.scm` + `script.img` Story Execution & Missing Gameplay Systems

1. **Extend `main.scm` & `script.img` Opcode Dispatch (`crates/gta_formats`, `crates/scripting`, `crates/missions`, `crates/game`)**:
   - Inspect `E:\gtasa\data\script\script.img` and wire external script loading + `LOAD_AND_LAUNCH_MISSION_INTERNAL` (`0x0417`), scripted cameras (`SET_FIXED_CAMERA_POSITION`, `POINT_CAMERA_AT_POINT`, `RESTORE_CAMERA_JUMPCUT`), scripted char/car creation & mission/blip/timer cleanup toward early Los Santos missions (*In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood*).
   - Add cutscene `.cut` / timed subtitle & actor choreography support.
2. **Deepen Vehicle, Combat & Open-World Systems (`crates/vehicles`, `crates/ai`, `crates/weapons`, `crates/audio`)**:
   - Motorbike/BMX lean, helicopter rotor spin & aircraft lift, Pay 'n' Spray / Mod Garages, gang territory wars, and procedural engine RPM/siren audio.
3. **Verify & Ship Milestone G2**:
   - Maintain full Windows x86_64 + WSL2 Linux x86_64 parity (`cargo fmt`, `clippy -- -D warnings`, `cargo test`, `--smoke-test`, live Vulkan launch), keep all 5 docs synchronized, and commit + push to `main`.

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
