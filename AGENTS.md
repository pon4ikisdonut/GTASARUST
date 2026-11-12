# AGENTS.md — Autonomous Development Guide for GTASARUST

This repository is an autonomous, AI-driven, open-source Rust recreation of *Grand Theft Auto: San Andreas* (PC DVD v1.00 baseline).

> **Starting or resuming a session?**
> Read **`CONTINUE.md`** immediately after this file to inspect the exact current milestone, last verified commit, and next concrete implementation steps.

---

## 1. Core Project Rules

1. **Full Playable Game Only**: Every subsystem (world, physics, rendering, vehicles, AI, weapons, audio, missions, scripting, saves, UI) must integrate into the live `gtasarust` game loop rather than staying an isolated stub or viewer.
2. **No Rockstar Assets in Git**: Never commit `.img`, `.dff`, `.txd`, `.col`, `.ifp`, `.scm`, `.gxt`, `.exe`, `.dll`, `.mp3`, `.ogg`, `.wav`, or any extracted game data. All original assets remain external and read-only (`E:\gtasa` on Windows / `/mnt/e/gtasa` on WSL2 Linux).
3. **Clean-Room Safety**: `crates/gta_formats` enforces `#![forbid(unsafe_code)]`. All archive/binary readers must validate bounds and reject malformed inputs cleanly without panicking. Always use case-insensitive resolution (`GameDataRoot` / `resolve_ci_*`) for files inside `--game-dir` so Linux case-sensitive filesystems match Windows paths.
4. **Architecture Support (`x86_64` ONLY)**: Support `x86_64-pc-windows-msvc` (Windows 10/11) and `x86_64-unknown-linux-gnu` (Ubuntu/Debian/Fedora/Arch). Never introduce 32-bit targets.
5. **Frequent Commits & Pushes**: Commit and push to `main` using Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`) after every verified milestone. **Before every commit & push**, update `CONTINUE.md`, `DEVELOPMENT_STATUS.md`, `FEATURE_MATRIX.md`, `CHANGELOG.md`, and `BUGS.md`.

---

## 2. Mandatory Verification Gate (Windows + Linux x86_64)

GitHub Actions runner billing may be unavailable on the repository account, so **local verification on both Windows and Linux WSL2 is mandatory before pushing to `main`**:

### Windows (`x86_64-pc-windows-msvc`, PowerShell UTF-8)
```powershell
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Set-Location E:\prjects\GTASARUST
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p gtasarust-game
.\target\release\gtasarust.exe --smoke-test --game-dir E:\gtasa
```
Also verify a timed windowed GPU launch of `.\target\release\gtasarust.exe --game-dir E:\gtasa` whenever rendering, UI, physics, or audio initialization changes.

### Linux (`x86_64-unknown-linux-gnu` via WSL2)
Remember to use a Linux-native `CARGO_TARGET_DIR` (e.g. `$HOME/gtasarust-linux-target`) so it never collides with Windows build artifacts or NTFS case sensitivity:
```bash
export CARGO_TARGET_DIR="$HOME/gtasarust-linux-target"
cd /mnt/e/prjects/GTASARUST
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
cargo test --workspace --target x86_64-unknown-linux-gnu
cargo build --release --workspace --target x86_64-unknown-linux-gnu
"$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/gtasarust" --headless --smoke-test --game-dir /mnt/e/gtasa
"$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/gtasarust-map-inspector" --game-dir /mnt/e/gtasa --include-binary-ipls
```

---

## 3. Workspace Architecture (24 Crates)

- `crates/gta_formats`: Clean-room SA format parsers (`img`, `rw` DFF/TXD, `col`, `ide`, `ipl`, `dat`, `paths` `NODES*.DAT`, `audio_cfg` streams & SFX banks, `ifp`, `scm`, `gxt`).
- `crates/engine`: Platform `x86_64` checks, `GameDataRoot` case-insensitive VFS, TOML config, `FixedTimestep`, `GameClock`.
- `crates/asset_pipeline`: Batch asset inspection, mesh/manifest conversion, error reporting.
- `crates/renderer`: `wgpu` + WGSL 3D renderer, timecycle, perspective camera, frustum culling, `DffSubMesh` instancing, `water.dat` mesh.
- `crates/physics`: `rapier3d` world, terrain/building `.col` colliders, ground raycasts, dynamic collision streaming.
- `crates/audio`: `kira` mixer, background Ogg Vorbis radio streaming (`audio/streams/*`), SFX bank playback (`audio/SFX/*`), procedural fallback SFX.
- `crates/input`: `winit` + `gilrs` keyboard/mouse/gamepad action mapping (`OnFoot`, `InVehicle`, `Menu`).
- `crates/world`: `gta.dat`/`default.dat` world loader, sector streaming, `.col` model index, `PathGraph`, zone lookup, weather.
- `crates/entities`: Player (CJ), on-foot locomotion w/ wall sliding, health/armor/stamina/wanted state.
- `crates/vehicles`: Vehicle physics, `handling.cfg` parameters, seat entry/exit, bumper crash damage.
- `crates/weapons`: `weapon.dat` definitions, 13-slot weapon inventory, firing/reloading.
- `crates/ai`: Sidewalk pedestrian AI, `NODES*.DAT` road traffic AI w/ yielding, carjacking crime/panic, 6-star police system.
- `crates/animation`: IFP skeletal hierarchy & animation state machine.
- `crates/scripting`: `main.scm` bytecode VM & opcode dispatch.
- `crates/missions`: Campaign mission triggers, checkpoints, progression.
- `crates/localization`: `GXT` lookup table w/ SA formatting tag stripping.
- `crates/ui`: `egui` menus, circular radar, SA HUD cluster, settings, save/load slots, `F3` diagnostics overlay.
- `crates/save_system`: Versioned JSON + SHA-256 save slots.
- `crates/game`: Main `gtasarust` binary (`main.rs` + `simulation.rs`).
- `crates/tools` + `tools/*`: CLI tools (`gtasarust-asset-extractor`, `gtasarust-asset-converter`, `gtasarust-map-inspector`, `gtasarust-model-viewer`).

---

## 4. Verified Format Invariants (`E:\gtasa` v1.00)

Do not re-investigate solved binary offsets:
- **Binary `bnry` IPL**: `cars_offset` is at byte offset **`0x3C`** (yields `1,043` `CARS` entries in `gta3.img`).
- **`NODES*.DAT`**: 64 files in `gta3.img` across an `8×8` grid of `750m` areas (`68,237` total nodes: `30,587` vehicle + `37,650` ped, `143,622` links).
- **RenderWare DFF/TXD**: Clump versions `0x34003`/`0x36003`; tristrip submeshes use alternating odd/even winding parity; primary car paint material color is `~(60, 255, 0)` and secondary is `~(255, 0, 175)`.
- **`audio/streams/*`**: Rotating 16-byte XOR key `[0xEA, 0x3A, 0xC4, 0xA1, 0x9A, 0xA8, 0x14, 0xF3, 0x48, 0xB0, 0xD7, 0x23, 0x9D, 0xE8, 0xFF, 0xF1]`; every `TrakLkup.dat` Ogg Vorbis stream begins at `entry.offset + 8068` (`SA_STREAM_HEADER_BYTES`). Guard against 0-byte placeholder stream files (`AA`, `CO`, `CR`, `DS`, `HC`, `MH`, `MR`, `RE`, `RG`, `TK`).
- **`audio/SFX/*`**: Each bank in `BankLkup.dat` starts with a fixed **`4804`-byte** header (`u16 num_sounds` + 400 × 12-byte slots `<IIHh>`), followed by uncompressed 16-bit LE signed mono PCM.
