# Known Bugs & Issues

This file tracks active defects, regressions, and platform issues discovered during development and QA.

## Active Issues

| ID | Severity | Subsystem | Summary | Status |
| --- | --- | --- | --- | --- |
| — | — | — | No open blockers in Milestone F1+F2 baseline (verified 46/46 workspace tests across Windows & WSL2 Linux + Vulkan RX 480 launch; remaining parity work is full mission/cutscene scripting, advanced ped/vehicle sub-classes & audio streams). | Clear |

## Resolved Issues

| ID | Severity | Subsystem | Summary | Resolution |
| --- | --- | --- | --- | --- |
| BUG-012 | High | `renderer` / `vehicles` | SA `.dff` vehicles only attach a single `wheel` atomic to `wheel_rf_dummy`, leaving 3 wheels missing and flat wheel spin/roll. | Implemented `CpuMeshData::dff_vehicle_parts` + `CachedVehicleModel` mounting rotated/steered/spun wheels onto all `wheel_*_dummy` frames (`wheel_lf_dummy` flipped `PI`) with 4-point `.col` pitch/roll alignment. |
| BUG-011 | High | `entities` / `renderer` / `game` | Turning the camera spun CJ on the spot, cursor was unlocked on Windows/Linux, and close buildings clipped into camera view. | Decoupled `camera_yaw`/`camera_pitch`/`camera_distance` from CJ's `yaw`, added camera-relative movement & aiming turn, pointer lock (`Locked`/`Confined`) w/ wheel zoom, and `.col` raycast camera collision (`update_third_person_collided`). |
| BUG-010 | High | `gta_formats` / `renderer` | Double-sRGB darkening, missing mipmap shimmer, opaque `DXT1` `565` alpha holes, ignored `txdp` parents & external `alpha_name` masks, and missing night prelit buffers. | Uploaded `Rgba8Unorm` w/ full 2x2 mipmaps & WGSL linear-to-sRGB encode, added `ExtraVertColour` (`0x0253F2F9`) + `timecyc.dat` `dn_balance`/`ambient_obj`/`water_color`, fixed `FORMAT_565` DXT1, composited `alpha_name` masks (`merge_rgba_alpha_mask`), wired `txdp` chains, and split 3 render passes via `IDE_FLAG_*`. |
| BUG-009 | High | `ui` / `game` | Hardcoded colored rectangles for radar/menus/fist and stubbed `main.scm` VM. | Wired `LOADSCS.TXD` (`loadsc0`), `hud.txd` (`69` sprites), `144` `radar00..radar143.txd` minimap + 12x12 fullscreen map, modular `player.img` CJ (`parts=5`), and real `ScmImage` chunk parser + `ScmVm` bytecode execution (`main_offset=55976`, `699` SA opcodes). |
| --- | --- | --- | --- | --- |
| BUG-008 | Medium | `gta_formats` / `world` | `IplSection::Enex` skipped field index 6 and read `exit`, `exit_rot`, `target_interior`, `flags`, and `name` shifted by one column. | Corrected field slice mapping to `fields[7..10]` (`exit`), `[10]` (`exit_rot`), `[11]` (`target_interior`), `[12]` (`flags`), `[13]` (`name`), and verified `CARLS` unit test (`target_interior=3`). |
| BUG-007 | Medium | `gta_formats` / `animation` | `RW_EXTENSION` frame indexing inside `parse_frame_list` misaligned `HAnim` node IDs across frames. | Tracked `ext_frame_idx` once per extension chunk inside `rw.rs` and verified full `fam1.dff` + `ped.ifp` CPU skinning (`1,011` moved vertices). |
| BUG-001 | Low | `gta_formats` / Audio | 0-byte placeholder radio stream files (`AA`, `CO`, `CR`, `DS`, `HC`, `MH`, `MR`, `RE`, `RG`, `TK`) in local SA install (`E:\gtasa`). | Handled gracefully in `asset_pipeline` inspector (emits non-fatal warnings) and `AudioEngine`. |
| BUG-002 | High | `ui` | `egui 0.37` shipped without `default_fonts` in default features, causing empty font provider errors at runtime. | Enabled `default_fonts` feature on `egui` in root `Cargo.toml`; verified zero UI font errors on launch. |
| BUG-003 | Low | `game` | `run_headless_smoke` reported `ticks=0` because it bypassed `FixedTimestep::accumulate`. | Updated `run_headless_smoke` to track `timestep.total_ticks` across all 60 ticks. |
| BUG-004 | Medium | `world` / `gta_formats` | Text/binary IPL car generator (`CARS`) parsing currently counts 0 entries in inspector manifest summary and does not yet spawn ambient parked traffic from IPLs. | Fixed `bnry` `cars_offset` at `0x3C` in `ipl.rs`, scanned `gta3.img` IPLs in `asset_pipeline` inspector (`1,045` car spawns / `1,043` binary `gta3.img`), printed `Car Spawns` in `map-inspector`, and spawned nearby parked vehicles at startup. |
| BUG-005 | Low | `renderer` / `world` | Preloaded DFF world meshes rendered with flat white material rather than resolved TXD textures, and tristrip winding/frame hierarchy needed fixes. | Implemented `DffSubMesh` per-material TXD texture resolution, hierarchical `DffFrame` transforms, vehicle primary/secondary paint tinting, `water.dat` GPU mesh, and alternating tristrip parity in `renderer`, `asset_pipeline`, and `game`. |
| BUG-006 | Medium | `physics` / `world` | `.col` collision archives are parsed and indexed, but static `rapier3d` terrain/building colliders are not yet registered in `PhysicsWorld` (player/vehicles spawn at `Z=0.0`). | Indexed `10,160` `.col` models in `WorldManager`, streamed transformed `rapier3d` trimesh/box/sphere colliders (`25,041` active in Grove Street), added wall sliding & vehicle bumper impacts, and grounded initial spawns (`Z=12.712m`). |
