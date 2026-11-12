# Security Policy

## Scope

GTASARUST parses complex binary game files (archives, models, textures, scripts, save files) provided locally by the user. Memory safety, bounds checking, and resilient parsing are top priorities.

## Parser Safety Guarantees

- All binary format parsers in `crates/gta_formats` and `crates/save_system` must treat input bytes as untrusted:
  - No `unsafe` pointer casting over unvalidated file buffers.
  - Explicit bounds checks and checked arithmetic for offsets, counts, and allocations.
  - Clear, structured errors instead of panics on malformed or truncated data.
- Save files must validate headers, schema versions, and checksums before loading state.

## Reporting a Vulnerability

If you discover a security vulnerability (such as an out-of-bounds panic, memory exhaustion vector via malformed assets/saves, or arbitrary path traversal during asset extraction):

1. **Do not** open a public GitHub issue with exploit details before a fix is available.
2. Report it via GitHub Security Advisories (`Security` -> `Advisories` -> `Report a vulnerability`) on `pon4ikisdonut/GTASARUST`.
3. Include reproduction steps, affected crate/version, and impact assessment.

We aim to acknowledge reports within 48 hours and publish a patched release promptly.
