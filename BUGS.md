# Known Bugs & Issues

This file tracks active defects, regressions, and platform issues discovered during development and QA.

## Active Issues

| ID | Severity | Subsystem | Summary | Status |
| --- | --- | --- | --- | --- |
| — | — | — | No open blockers in Phase 4/6 baseline (next targets: `NODES*.DAT` traffic AI, `audio/streams` Vorbis decoding, `ped.ifp` skeletal playback). | Clear |

## Resolved Issues

| ID | Severity | Subsystem | Summary | Resolution |
| --- | --- | --- | --- | --- |
| BUG-001 | Low | `gta_formats` / Audio | 0-byte placeholder radio stream files (`AA`, `CO`, `CR`, `DS`, `HC`, `MH`, `MR`, `RE`, `RG`, `TK`) in local SA install (`E:\gtasa`). | Handled gracefully in `asset_pipeline` inspector (emits non-fatal warnings) and `AudioEngine`. |
| BUG-002 | High | `ui` | `egui 0.37` shipped without `default_fonts` in default features, causing empty font provider errors at runtime. | Enabled `default_fonts` feature on `egui` in root `Cargo.toml`; verified zero UI font errors on launch. |
| BUG-003 | Low | `game` | `run_headless_smoke` reported `ticks=0` because it bypassed `FixedTimestep::accumulate`. | Updated `run_headless_smoke` to track `timestep.total_ticks` across all 60 ticks. |
| BUG-004 | Medium | `world` / `gta_formats` | Text/binary IPL car generator (`CARS`) parsing currently counts 0 entries in inspector manifest summary and does not yet spawn ambient parked traffic from IPLs. | Fixed `bnry` `cars_offset` at `0x3C` in `ipl.rs`, scanned `gta3.img` IPLs in `asset_pipeline` inspector (`1,045` car spawns / `1,043` binary `gta3.img`), printed `Car Spawns` in `map-inspector`, and spawned nearby parked vehicles at startup. |
| BUG-005 | Low | `renderer` / `world` | Preloaded DFF world meshes rendered with flat white material rather than resolved TXD textures, and tristrip winding/frame hierarchy needed fixes. | Implemented `DffSubMesh` per-material TXD texture resolution, hierarchical `DffFrame` transforms, vehicle primary/secondary paint tinting, `water.dat` GPU mesh, and alternating tristrip parity in `renderer`, `asset_pipeline`, and `game`. |
| BUG-006 | Medium | `physics` / `world` | `.col` collision archives are parsed and indexed, but static `rapier3d` terrain/building colliders are not yet registered in `PhysicsWorld` (player/vehicles spawn at `Z=0.0`). | Indexed `10,160` `.col` models in `WorldManager`, streamed transformed `rapier3d` trimesh/box/sphere colliders (`25,041` active in Grove Street), added wall sliding & vehicle bumper impacts, and grounded initial spawns (`Z=12.712m`). |
