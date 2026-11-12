# CONTINUE.md — GTASARUST Live Handoff & Resume State

> **Notice to any resuming AI agent:** Read `AGENTS.md` and this file first, then execute the **Current Next Task** directly without re-auditing completed phases unless tests fail. Update this file and commit + push to `main` after every verified milestone.

---

## 1. Current Verified State (Post-Commit D Baseline)

- **Repository**: `E:\prjects\GTASARUST` (`https://github.com/pon4ikisdonut/GTASARUST.git`, branch `main`)
- **Last Completed Milestone**: **Milestone D — Real GTA:SA Audio Streams (`OggS`) & SFX Bank (`PCM`) Playback + Agent Handoff Docs**
- **Previous Verified Commits on `main`**:
  - `dd47d1d` — Initial verified 24-crate Rust workspace on Windows + Linux x86_64
  - `7d5e8c9` — **Milestone A**: Textured RenderWare DFF/TXD per-submesh rendering, `DffFrame` hierarchy, alternating tristrip winding parity, vehicle paint tinting, `water.dat` 301-quad mesh, sector asset streaming (`BUG-005` fixed)
  - `5e19b6e` — **Milestone B**: `.col` collision streaming (`10,160` models / `25,041` active `rapier3d` colliders at Grove Street), on-foot wall sliding & spawn grounding (`Z=12.712m`), vehicle crash damage, binary `bnry` IPL `cars_offset` fix at `0x3C` (`1,043` `CARS`), parked traffic (`BUG-004`, `BUG-006` fixed)
  - `3f5c0f5` — **Milestone C**: All 64 `NODES*.DAT` road/ped path graphs (`68,237` nodes / `143,622` links) driving sidewalk pedestrians, moving ambient road vehicles w/ obstacle yielding, and carjacking crime/panic
  - **Milestone D (current commit)**:
    - `crates/gta_formats/src/audio_cfg.rs`: `AudioStreamIndex` (`StrmPaks.dat` + `TrakLkup.dat`, rotating 16-byte XOR decoder, `OggS` extraction at `offset + 8068`) and `SfxBankIndex` (`PakFiles.dat` + `BankLkup.dat`, 4804-byte bank header, 16-bit LE mono PCM extraction), plus unit tests
    - `crates/audio/src/lib.rs`: `kira 0.12.6` integration — indexes `active_stations=7`, `total_tracks=264`, `sa_sfx_loaded=10` (`4` `FEET[0]` footsteps + `GENRL` gunshots/punches/horn/crash/radio click), spawns non-blocking background Ogg Vorbis decode threads, switches radio on `R` and auto-tunes on vehicle entry, mutes radio on exit, provides procedural fallback SFX when `--game-dir` is omitted
    - `crates/game/src/simulation.rs`: Attaches `--game-dir` to `AudioEngine`, drives `audio.update(dt)`, plays `Footstep`, weapon-appropriate `PistolShot`/`HeavyShot`/`Punch`, `CarHorn`, `Crash`, and validates full Ogg Vorbis decode in `--smoke-test` (`decoded_radio=Some(("BEATS", 6597202, 32000))`)
    - Verified across **Windows 10/11 x86_64** (`fmt`, `clippy -- -D warnings`, `cargo test --workspace`, `--release` build, `--smoke-test --game-dir E:\gtasa`, 4s Vulkan Radeon RX 480 launch) and **WSL2 Ubuntu 22.04 x86_64** (`linux_verify.sh`: `fmt`, `clippy`, `test`, `release`, headless smoke, `gtasarust-map-inspector`)

- **Open Blockers**: None (`BUG-001`..`BUG-006` all resolved in `BUGS.md`).

---

## 2. Current Next Task — Milestone E: Skeletal `ped.ifp` Animation, Character Rendering & `ENEX` Interiors

Implement Milestone E in order, integrating everything into `gtasarust`:

1. **`crates/gta_formats/src/ifp.rs` + `crates/animation/src/lib.rs`**:
   - Parse full `ANPK` / `ANP3` bone keyframes (`KR00` / `KRT0` / `KRTS` root translations + quantized `i16 / 4096.0` quaternion rotations + frame timestamps) in `ped.ifp` (already `294` clips indexed by header; load per-bone animation tracks + unit tests on real `ped.ifp` header/synthetic `ANP3` buffer).
   - Add bone pose evaluation (`evaluate_pose(clip, time_sec, looped)`) inside `AnimationController` for idle, walk, run/sprint, jump, crouch, punch/shoot, and seated drive states.
2. **Articulated CJ & Pedestrian Rendering (`crates/renderer`, `crates/game/src/simulation.rs`)**:
   - Load real `player.img` / `gta3.img` pedestrian DFF/TXD models (e.g. `player` / `fam1` / `fam2` / `bmydj` / `lapd1`) or multi-part hierarchical bone transforms driven by `AnimationController` rather than static capsules, updating active character submesh transforms each frame in `simulation.rs`.
3. **`ENEX` Interior Markers, Shops & Safehouse Saves (`crates/gta_formats/src/ipl.rs`, `crates/world/src/lib.rs`, `crates/game/src/simulation.rs`, `crates/ui/src/lib.rs`)**:
   - Ensure IPL `enex` (entrance/exit) records are preserved in `WorldManager` with entrance position, target interior coordinates, target interior ID, and marker name (starting with Johnson House safehouse on Grove Street, Binco/SubUrban/24-7/Cluckin' Bell/Ammu-Nation in Los Santos).
   - Render iconic yellow interior entry cones in `simulation.rs`, support walking into `ENEX` markers to transition between exterior (`interior=0`) and interior cells (filtering `IplInstance::interior` sector rendering and reloading collision/placements for the active interior!), add safehouse save trigger at Johnson House, and let Ammu-Nation/Cluckin' Bell spend `player.money` for weapons/food health.
4. **Verify & Ship Milestone E**:
   - Run full Windows verification + WSL2 Linux verification (`linux_verify.sh`).
   - Update `CONTINUE.md` (advance Next Task to Milestone F), `DEVELOPMENT_STATUS.md`, `FEATURE_MATRIX.md`, `CHANGELOG.md`.
   - `git commit` & `git push origin main`.

---

## 3. Subsequent Roadmap (After Milestone E)

- **Milestone F — `main.scm` Bytecode VM & Campaign Missions**:
  - Expand `crates/scripting/src/lib.rs` to execute real SCM opcodes (`0001` WAIT, `0002` GOTO, `00D6` IF, player/actor/vehicle spawn & marker/blip opcodes, `CREATE_THREAD`, GXT subtitle/brief queue `00BA`/`00BC`).
  - Wire Grove Street early missions (*In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, *Cleaning the Hood*) seamlessly with live world physics, AI, UI mission text, and save persistence.
- **Milestone G — Full San Andreas World Completeness & Systems**:
  - Vehicle classes (motorcycles/BMX lean, boats + water buoyancy on `water.dat`, helicopters/planes rotor & lift), vehicle weapons/drive-by, police roadblocks/helicopters at 3–6 stars, mod garages / Pay 'n' Spray, gang warfare zones, and full Los Santos -> San Fierro -> Las Venturas mission arc progression.

---

## 4. Quick Environment Notes for Resuming Agents

- **Game Installation Path**:
  - Windows: `E:\gtasa`
  - WSL2 Linux: `/mnt/e/gtasa`
- **GitHub Action Runner Note**: Remote CI workflows may fail immediately with account runner billing disabled; always verify Linux `x86_64-unknown-linux-gnu` locally inside WSL2 using `CARGO_TARGET_DIR="$HOME/gtasarust-linux-target"` before pushing.
- **Last Windows & Linux Smoke Benchmark Output** (for regression comparison):
  - Map Inspector: `Objects: 14344, Vehicles: 198, Peds: 275, Sectors: 1455, Total Placements: 45837, Car Spawns: 1043, Path Nodes: 68237, Zones: 384, Water Quads: 301`
  - Smoke `--game-dir E:\gtasa`: `ticks=60 player_pos=Vec3(2503.5032, -1688.2, 0.0) zone=LA01 vehicles=3 handling_entries=210 audio_stations=7 audio_sfx_clips=10 decoded_radio=Some(("BEATS", 6597202, 32000))`
  - GPU Window Launch: `player_pos=Vec3(2495.3, -1688.2, 12.712079) colliders=25041 vehicles=8 models=71 textures=639` (`Vulkan Radeon RX 480`, `Kira audio backend initialized`).
