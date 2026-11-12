# CONTINUE.md — GTASARUST Live Handoff & Resume State

> **Notice to any resuming AI agent:** Read `AGENTS.md` and this file first, then execute the **Current Next Task** directly without re-auditing completed phases unless tests fail. Update this file and commit + push to `main` after every verified milestone.

---

## 1. Current Verified State (Post-Commit E Baseline)

- **Repository**: `E:\prjects\GTASARUST` (`https://github.com/pon4ikisdonut/GTASARUST.git`, branch `main`)
- **Last Completed Milestone**: **Milestone E — `ped.ifp` Skeletal Animation, Articulated CJ/Ped Rendering & `ENEX` Interiors**
- **Previous Verified Commits on `main`**:
  - `dd47d1d` — Initial verified 24-crate Rust workspace on Windows + Linux x86_64
  - `7d5e8c9` — **Milestone A**: Textured RenderWare DFF/TXD per-submesh rendering, `DffFrame` hierarchy, alternating tristrip winding parity, vehicle paint tinting, `water.dat` 301-quad mesh, sector asset streaming (`BUG-005` fixed)
  - `5e19b6e` — **Milestone B**: `.col` collision streaming (`10,160` models / `25,041` active `rapier3d` colliders at Grove Street), on-foot wall sliding & spawn grounding (`Z=12.712m`), vehicle crash damage, binary `bnry` IPL `cars_offset` fix at `0x3C` (`1,043` `CARS`), parked traffic (`BUG-004`, `BUG-006` fixed)
  - `3f5c0f5` — **Milestone C**: All 64 `NODES*.DAT` road/ped path graphs (`68,237` nodes / `143,622` links) driving sidewalk pedestrians, moving ambient road vehicles w/ obstacle yielding, and carjacking crime/panic
  - `3f4a1e0` — **Milestone D**: Real `audio/streams` XOR-unscrambled Ogg Vorbis radio (`264` tracks across `7` stations) & `audio/SFX` 16-bit PCM bank playback (`10` clips from `FEET`/`GENRL`) via `kira 0.12.6`, plus `AGENTS.md` / `CONTINUE.md`
  - **Milestone E (current commit)**:
    - `crates/gta_formats/src/{ifp.rs,rw.rs,ipl.rs}`: Full `ANP3` `ped.ifp` bone keyframe decoder (`IfpArchive::parse_full`, `i16/4096.0` non-conjugated quaternions & `i16/1024.0` root translations at `t/60.0`), `RW_SKIN_PLG` (`0x0116`) bone weights/indices/`inv_bind_matrices` & `RW_HANIM_PLG` (`0x011E`) node mappings w/ `RW_EXTENSION` frame-index fix (`BUG-007`), and `ENEX` off-by-one field fix (`fields[7..14]`, `BUG-008`)
    - `crates/animation/src/lib.rs`: `IfpPoseLibrary`, `AnimationController::sample_poses`, and hierarchical `compute_skin_matrices` evaluator across `294` `ped.ifp` clips
    - `crates/renderer/src/{mesh.rs,renderer.rs}`: `SkinnedVertexWeights`, `DffSubMesh::compute_skinned_vertices`, `CpuMeshData::marker_cone`, dynamic `BufferUsages::VERTEX | BufferUsages::COPY_DST` GPU mesh creation (`upload_dynamic_mesh`) and per-frame updates (`update_mesh_vertices`)
    - `crates/world/src/lib.rs`, `crates/entities/src/lib.rs`, `crates/save_system/src/lib.rs`, `crates/ui/src/lib.rs`, `crates/game/src/simulation.rs`: `gta_int.img` binary IPL loading (`50,935` total placements across `1,457` sectors, `376` `ENEX` portals), dynamic CPU-skinned CJ (`fam1`, `1011` verts moved under `walk_civi`) and NPCs (`fam2`/`fam3`/`bmydj`/`lapd1`/`hmost`), floating yellow `ENEX` marker cones + radar doorway blips + help toast HUD, bidirectional interior transitions w/ `.col` collider rebuild & floor guard, and interactive Johnson House (`CARLS` safehouse heal+auto-save), Cluckin' Bell (`FDCHICK` `-$10, +50 HP`), and Ammu-Nation (`AMMUN*` `-$200, +50` 9mm reserve) counters
    - Verified across **Windows 10/11 x86_64** (`fmt`, `clippy -- -D warnings`, `cargo test --workspace`, `--release` build, `--smoke-test --game-dir E:\gtasa`, 4s Vulkan Radeon RX 480 launch) and **WSL2 Ubuntu 22.04 x86_64** (`linux_verify.sh`: `fmt`, `clippy`, `test`, `release`, headless smoke, `gtasarust-map-inspector`)

- **Open Blockers**: None (`BUG-001`..`BUG-008` all resolved in `BUGS.md`).

---

## 2. Current Next Task — Milestone F: `main.scm` Bytecode VM & Grove Street Story Missions

Implement Milestone F in order, integrating everything into `gtasarust`:

1. **`crates/gta_formats` + `crates/scripting/src/lib.rs` (Real `main.scm` Bytecode Execution)**:
   - Parse `main.scm` header tables (global variables space, object/model name table, mission offset table, MAIN section bytecode stream) and support multi-thread execution (`CREATE_THREAD` `004F`, `WAIT` `0001`, `GOTO` `0002`, `GOSUB`/`RETURN`, `TERMINATE_THIS_THREAD` `004E`, arithmetic & comparison opcodes, conditional jumps `004D` `JF`/`IF`).
   - Dispatch core gameplay opcodes to engine state via host callbacks/commands: player spawn/freeze, actor/ped/vehicle spawn (`009A`, `00A5`), blip/sphere checkpoints, `00BA`/`00BC` GXT text-key big/subtitle banners, money/respect rewards (`0109`), and mission start/pass/fail progression.
2. **Playable Early Grove Street Campaign Integration (`crates/missions/src/lib.rs`, `crates/game/src/simulation.rs`)**:
   - Wire *In the Beginning*, *Big Smoke*, *Ryder*, *Tagging Up Turf*, and *Cleaning the Hood* cleanly into the world/HUD/save loop so starting at Grove Street triggers real GXT objectives, spawns quest NPCs/vehicles (Sweet's Greenwood, Ryder's Picador, BMX chase/spray tags/Ballas confrontation), reacts to player health/vehicle/weapon state, awards money/respect, unlocks next missions, and persists `completed_missions` cleanly through save slots (`slot 1..8`).
   - Add automated mission/SCM integration & unit tests in `gtasarust-scripting`, `gtasarust-missions`, and `tests/gameplay/gameplay_integration.rs`.
3. **Verify & Ship Milestone F**:
   - Run full Windows verification + WSL2 Linux verification (`linux_verify.sh`).
   - Update `CONTINUE.md` (advance Next Task to Milestone G), `DEVELOPMENT_STATUS.md`, `FEATURE_MATRIX.md`, `CHANGELOG.md`.
   - `git commit` & `git push origin main`.

---

## 3. Subsequent Roadmap (After Milestone F)

- **Milestone G — Full San Andreas World Completeness & Systems**:
  - Vehicle classes (motorcycles/BMX lean, boats + water buoyancy on `water.dat`, helicopters/planes rotor & lift), vehicle weapons/drive-by, police roadblocks/helicopters at 3–6 stars, mod garages / Pay 'n' Spray, gang warfare zones, and full Los Santos -> San Fierro -> Las Venturas mission arc progression.

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
