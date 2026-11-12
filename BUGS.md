# Known Bugs & Issues

This file tracks active defects, regressions, and platform issues discovered during development and QA.

## Active Issues

| ID | Severity | Subsystem | Summary | Status |
| --- | --- | --- | --- | --- |
| BUG-004 | Medium | `world` / `gta_formats` | Text/binary IPL car generator (`CARS`) parsing currently counts 0 entries in inspector manifest summary and does not yet spawn ambient parked traffic from IPLs. | Open |
| BUG-005 | Low | `renderer` / `world` | Preloaded DFF world meshes render with flat white material rather than resolved TXD textures, and COL files are not yet registered into `rapier3d` colliders. | Open |

## Resolved Issues

| ID | Severity | Subsystem | Summary | Resolution |
| --- | --- | --- | --- | --- |
| BUG-001 | Low | `gta_formats` / Audio | 0-byte placeholder radio stream files (`AA`, `CO`, `CR`, `DS`, `HC`, `MH`, `MR`, `RE`, `RG`, `TK`) in local SA install (`E:\gtasa`). | Handled gracefully in `asset_pipeline` inspector (emits non-fatal warnings) and `AudioEngine`. |
| BUG-002 | High | `ui` | `egui 0.37` shipped without `default_fonts` in default features, causing empty font provider errors at runtime. | Enabled `default_fonts` feature on `egui` in root `Cargo.toml`; verified zero UI font errors on launch. |
| BUG-003 | Low | `game` | `run_headless_smoke` reported `ticks=0` because it bypassed `FixedTimestep::accumulate`. | Updated `run_headless_smoke` to track `timestep.total_ticks` across all 60 ticks. |
