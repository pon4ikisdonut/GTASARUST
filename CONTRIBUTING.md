# Contributing to GTASARUST

Thank you for your interest in contributing to **GTASARUST**.

> **Note:** GTASARUST is primarily developed and maintained by an autonomous AI engineering team. Human and AI contributions alike are welcome as long as they follow the project's clean-room engineering standards and licensing rules.

## Intellectual Property & Clean-Room Rules

1. **NEVER commit copyrighted Rockstar Games / Take-Two assets** (`.img`, `.dff`, `.txd`, `.col`, `.ifp`, `.scm`, `.gxt`, `.mpg`, `.exe`, `.dll`, audio banks, radio streams, or extracted data).

## Development Environment

- Target architecture: **x86_64 ONLY** (`x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`).
- Use platform-independent filesystem paths (`std::path::Path` / `PathBuf` + case-insensitive lookup helper for original data files on Linux).
- Do not hardcode Windows paths or drive letters in source code.

## Coding Standards

- Format all code with `cargo fmt --all`.
- Ensure zero warnings under `cargo clippy --workspace --all-targets -- -D warnings`.
- Write unit tests for parsers, physics/gameplay logic, and state transitions.
- Avoid allocations in hot render/simulation loops where practical.
- Update `DEVELOPMENT_STATUS.md`, `FEATURE_MATRIX.md`, `BUGS.md`, and `CHANGELOG.md` alongside feature or bugfix PRs.

## Dependency Policy

Before adding a new crate dependency to `Cargo.toml`, verify:
1. Active maintenance status.
2. Windows x64 and Linux x64 support.
3. Permissive open-source license compatibility (MIT / Apache-2.0 / BSD / Zlib).
4. Minimal transitive bloat and clean performance characteristics.

## Pull Request Process

1. Ensure `cargo fmt --check`, `cargo clippy`, and `cargo test --workspace` pass.
2. Verify cross-platform compilation (CI checks both Windows and Linux x86_64).
3. Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`).
4. Complete the Pull Request template checklist.
