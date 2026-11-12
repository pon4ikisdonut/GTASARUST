# CONTINUE.md — GTASARUST Live Handoff & Resume State

> **Notice to any resuming AI agent:** Read `AGENTS.md` and this file first, then execute the **Current Next Task** directly without re-auditing completed phases unless tests fail. Update this file and commit + push to `main` after every verified milestone.

---

## 1. Current Verified State (Milestone G2 Baseline)

- **Repository**: `E:\prjects\GTASARUST` (`https://github.com/pon4ikisdonut/GTASARUST.git`, branch `main`)
- **Last Completed Milestone**: **Milestone G2 — Retail `main.scm` + `script.img` Story/Cutscene VM, `.cut` Cutscenes & Subtitles, `grge` Pay 'n' Spray Garages, Helicopter Rotors, Sirens & Looping RPM Audio**
- **Previous Verified Commits on `main`**:
  - `dd47d1d` — Initial verified 24-crate Rust workspace on Windows + Linux x86_64
  - `7d5e8c9` — **Milestone A**: Textured RenderWare DFF/TXD per-submesh rendering, `DffFrame` hierarchy, alternating tristrip winding parity, vehicle paint tinting, `water.dat` 301-quad mesh, sector asset streaming (`BUG-005` fixed)
  - `5e19b6e` — **Milestone B**: `.col` collision streaming (`10,160` models / `25,041` active `rapier3d` colliders at Grove Street), on-foot wall sliding & spawn grounding (`Z=12.712m`), vehicle crash damage, binary `bnry` IPL `cars_offset` fix at `0x3C` (`1,043` `CARS`), parked traffic (`BUG-004`, `BUG-006` fixed)
  - `3f5c0f5` — **Milestone C**: All 64 `NODES*.DAT` road/ped path graphs (`68,237` nodes / `143,622` links) driving sidewalk pedestrians, moving ambient road vehicles w/ obstacle yielding, and carjacking crime/panic
  - `3f4a1e0` — **Milestone D**: Real `audio/streams` XOR-unscrambled Ogg Vorbis radio (`264` tracks across `7` stations) & `audio/SFX` 16-bit PCM bank playback (`10` clips from `FEET`/`GENRL`) via `kira 0.12.6`, plus `AGENTS.md` / `CONTINUE.md`
  - `282c342` — **Milestone E**: Full `ANP3` `ped.ifp` decoder (`294` clips), `SkinPLG`/`HAnim` CPU skinning (`fam1`, `1,011` verts deformed), `gta_int.img` binary IPLs (`50,935` placements / `376` `ENEX` portals), safehouse & Cluckin' Bell / Ammu-Nation counters (`BUG-007`, `BUG-008` fixed)
  - `06e45a1` — **Milestone F1+F2**: Fixed double-sRGB darkening & DXT1 565 alpha/txdp chains/3-pass `IDE_FLAG_*` rendering, `ExtraVertColour` day/night prelit buffers + `timecyc.dat` interpolation, modular `player.img` CJ, decoupled orbit camera w/ `.col` anti-clipping, `dff_vehicle_parts` 4-wheel mounts, retail `LOADSCS.TXD`/`hud.txd`/`radar00..143.txd` UI, `ScmImage` chunk parser + `ScmVm` (`699` SA opcodes)
  - `ba36856` — **Milestone G1**: Exact `water.dat` swimming/drowning + scrolling shader, dynamic per-vehicle `rapier3d` colliders + `< 250 HP` explosion lifecycle, OTS aiming camera + `BONE_R_HAND` weapon DFFs + occluded raycasts, `convert_shadow_luminance_to_alpha` shadows, `CHudColours` GXT rich text, `DoFade`/`DisplayHud`/`DisplayRadar`
  - **Milestone G2 (current commit)**:
    - **Retail Story, `script.img` & `.cut` Cutscenes (`crates/{gta_formats,scripting,localization,ui,input,game}`)**: Expanded `scm_opcodes.rs` to `1,964` SA opcode entries (`is_condition: bool`, `SwitchTable` 18-arg decode, `decode_instruction_slice`), `CutsceneFile::parse` (`cut.rs`), `ScmVm` `ScmCodeSource::{Main, Streamed(usize)}`, `1024`-local mission thread array, `TIMERA`/`TIMERB` (`locals[32..=33]`), `SWITCH_START`/`SWITCH_CONTINUED` (`0x0871`/`0x0872`), `DISPLAY_HUD` (`0x0826`), `HAS_*_LOADED`, streamed `.scm` loading (`79` retail `script.img` scripts), `GxtArchive::get_any`/`LocalizationTable::lookup_any`, scripted fixed camera overrides, widescreen black bars + timed GXT cutscene subtitles (`anim/cuts.img`) w/ `Space`/`Enter` skip (`skip_cutscene_just_pressed`), plus `22` retail `main.scm` threads verified surviving past `1,400` ticks
    - **`grge` Garages & Pay 'n' Spray (`crates/{gta_formats,world,ui,audio,game}`)**: Added `IplGarage` (`IplSection::Grge` 11-field parser in `ipl.rs`), `WorldManager::{garages, garage_center, is_pos_inside_garage, garage_at, pay_n_spray_garages}`, radar `radar_spray` icons & exterior glowing spray beacons, interactive Pay 'n' Spray (`garage_type == 5`) `$100` vehicle repair, repaint, wanted clear (`0` stars) & `SfxCue::Respray` audio
    - **Helicopters, Emergency Sirens & Vehicle Audio (`crates/{vehicles,renderer,audio,game}`)**: Added `VehicleEntity.rotor_spin`, helicopter flight pitch/roll tilt & spinning rotors, `CpuMeshData::dff_vehicle_parts` extraction of main/tail rotor submeshes & pivots (`static_rotor*`/`moving_rotor*`/`toprotor*`/`rearrotor*`), flashing red/blue emergency roof siren bars, continuous RPM-scaled looping engine audio & siren playback (`AudioEngine::update_vehicle_audio`, `SfxCue::Siren`, `SfxCue::Respray`)
    - Verified across **Windows 10/11 x86_64** (`62/62` workspace tests pass, `clippy --all-targets -- -D warnings`, `fmt --check`, release `--headless --smoke-test`, 5s live Vulkan RX 480 launch: `streamed_scripts=79 colliders=25049 models=359 textures=3054 backdrop=true hud_sprites=69 radar_tiles=144 sa_sfx_loaded=11`) and **WSL2 Ubuntu-22.04 x86_64** (`fmt --check`, `clippy --all-targets -- -D warnings`, `62/62` workspace tests pass, release `--headless --smoke-test`)

- **Open Blockers**: None (`BUG-001`..`BUG-015` resolved in `BUGS.md`).

---

## 2. Current Next Task — Milestone H1: Full Story Mission Progression, Planes/Bikes/Trains & Richer Audio/World Systems

1. **Drive Early Los Santos Story Missions End-to-End (`crates/{scripting,missions,ai,game}`)**:
   - Wire remaining task/actor/car opcodes encountered once CJ enters gameplay past the Jefferson alley intro (`PROLOG3` -> *Big Smoke*, *Sweet & Kendl*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood*): ped path/enter-car/follow scripts, bike chase AI, graffiti tag decals, shop/clothing/barber/tattoo wardrobe hooks, and mission-specific checkpoints/cleanup.
2. **Complete Remaining Vehicle Classes & Open-World Depth (`crates/{vehicles,ai,audio}`)**:
   - Airplane lift/flight physics + propeller spin (`misc_a`/`prop`), railway/tram spline paths, Mod Garages (TransFender/Loco Low Co.), safehouse garage vehicle storage persistence, gang territory wars, and radio DJ/commercial banter + ped speech SFX banks.
3. **Verify & Ship Milestone H1**:
   - Maintain full Windows x86_64 + WSL2 Linux x86_64 parity (`cargo fmt`, `clippy -- -D warnings`, `cargo test`, `--smoke-test`, live Vulkan launch), keep all 5 docs synchronized, and commit + push to `main`.

---

## 3. Subsequent Roadmap (After Milestone H1)

- **Milestone H2 — San Fierro, Las Venturas & Full Endgame Polish**:
  - Mid/late-game story arcs, casino/heist/jetpack mechanics, interior minigames, performance profiling, and PC release packaging.

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
