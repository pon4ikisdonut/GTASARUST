# GTASARUST Assets Directory

This directory contains **only engine-owned open-source assets**:

- `shaders/` — Original WGSL shaders written for GTASARUST (`wgpu`).
- `manifests/` — Generated asset schema/manifest templates (no copyrighted content).
- `fonts/` — Open-source fallback fonts or UI glyph metadata.
- `ui/` — Engine-owned UI layout definitions.

## Copyrighted GTA: San Andreas Assets

Original Rockstar Games assets (`.img`, `.dff`, `.txd`, `.col`, `.ifp`, `.scm`, `.gxt`, audio banks/streams, movies) are **NOT** stored in this repository and are excluded via `.gitignore`.

To use original assets locally:

```bash
# Inspect and validate your authorized GTA: San Andreas installation
cargo run -p gtasarust-asset-extractor -- inspect --game-dir "E:\gtasa"

# Extract & convert validated assets to a local (gitignored) directory
cargo run -p gtasarust-asset-extractor -- extract --game-dir "E:\gtasa" --out "assets/extracted"
```
