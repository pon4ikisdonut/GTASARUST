# Compatibility Matrix

GTASARUST targets **x86_64 / AMD64 ONLY**. 32-bit architectures are unsupported.

## Operating Systems

| Platform | Target Triple | Build Status | Runtime Test Status | Notes |
| --- | --- | --- | --- | --- |
| Windows 10/11 x64 | `x86_64-pc-windows-msvc` | Passing (Release & Debug) | Passing (`cargo test --workspace`, headless smoke test, 4s Vulkan RX 480 windowed launch) | Primary host development environment (`E:\gtasa` v1.00 tested) |
| Ubuntu 22.04+ x86_64 | `x86_64-unknown-linux-gnu` | Passing (Release, Clippy `-D warnings`) | Passing (native WSL2 Ubuntu 22.04.5 LTS: `cargo test --workspace`, headless smoke test + `map-inspector` against `/mnt/e/gtasa`) | Verified native Linux binary, ALSA/udev/X11/Wayland linkage & case-insensitive VFS |
| Debian x86_64 | `x86_64-unknown-linux-gnu` | Pending | Pending | Uses standard glibc + Vulkan/X11/Wayland |
| Fedora x86_64 | `x86_64-unknown-linux-gnu` | Pending | Pending | Uses standard glibc + Vulkan/X11/Wayland |
| Arch Linux x86_64 | `x86_64-unknown-linux-gnu` | Pending | Pending | Uses standard glibc + Vulkan/X11/Wayland |

## Inspected Original Game Editions

| Edition | Executable SHA-256 | Verified Notes |
| --- | --- | --- |
| PC DVD/Retail v1.00 (`E:\gtasa\gta_sa.exe`, 14,383,616 bytes) | `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26` | Contains full `models/*.img`, `data/maps/*`, `data/script/main.scm`, `text/*.gxt`, `anim/*.ifp`, `audio/SFX/*`; partial `audio/streams/*` |

## Cross-Platform Path Handling

Original `data/gta.dat` and `data/default.dat` reference files using Windows-style backslashes and inconsistent casing (e.g., `DATA\MAPS\LA\LAwn.IDE` vs `data\maps\LA\LaWn.ide`). GTASARUST's virtual filesystem resolver normalizes separators and performs case-insensitive segment matching on case-sensitive Linux filesystems.
