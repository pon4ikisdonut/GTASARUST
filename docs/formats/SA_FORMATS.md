# Original GTA: San Andreas Formats (Clean-Room Reference)

Inspected against a local PC v1.00 installation (`E:\gtasa`):

## Archives (`models/*.img`, `anim/*.img`, `data/script/script.img`)
- **IMG Version 2 (`VER2`)**:
  - Header: `4` bytes magic `"VER2"`, `u32` entry count.
  - Directory table: `entry_count * 32` bytes: `u32 offset_sectors`, `u16 streaming_size_sectors`, `u16 size_in_archive_sectors`, `24`-byte null-padded ASCII name (`sector = 2048` bytes).

## RenderWare Binary Streams (`.dff`, `.txd`)
- Standard RW chunk header: `u32 chunk_type`, `u32 byte_size`, `u32 rw_version` (SA PC uses `0x1803FFFF` = RW 3.6.0.3).
- **TXD (`0x0016` TexDictionary)**: Contains `Struct` (texture count) + `TextureNative` (`0x0015`) chunks with PC D3D8/D3D9 rasters (`DXT1`, `DXT3`, `DXT5`, `A8R8G8B8`, `X8R8G8B8`, `R5G6B5`, `A1R5G5B5`, palettized `PAL8`/`PAL4`).
- **DFF (`0x0010` Clump)**: Contains `FrameList`, `GeometryList` (with `BinMeshPLG` triangle lists/strips, UV sets, vertex colors, normals, materials, textures), and `Atomic` chunks.

## Collision Files (`.col`)
- Multi-model container supporting `COLL` (v1), `COL2` (v2), `COL3` (v3, SA standard), and `COL4` (v4): bounding sphere/box, collision spheres, boxes, compressed/float vertices, triangle faces with surface material IDs, and optional shadow meshes.

## World Definition & Placement (`.ide`, `.ipl`)
- **IDE**: Sections `objs`, `tobj`, `anim`, `peds`, `cars`, `hier`, `txdp`, `weap`, `2dfx`, `path`, ending with `end`.
- **IPL (Text)**: Sections `inst`, `cull`, `zone`, `pick`, `occl`, `grge`, `enex`, `cars`, `auzo`, `jump`, `tcyc`.
- **IPL (Binary `bnry`)**: Found inside `gta3.img` as `*_streamN.ipl`; header with instance count and packed binary `inst`/`cars` records.

## Localization (`.gxt`)
- SA PC uses `TABL` (table directory) + per-table `TKEY` (CRC32-keyed entries -> offset) + `TDAT` (null-terminated Windows-1252 / extended ASCII strings with `~r~`, `~g~`, `~b~`, `~w~`, `~y~`, `~n~` formatting tags).

## Mission Scripts (`main.scm`, `script.img`)
- `main.scm`: Global variable block + model table + mission offset table + multi-script SCM bytecode stream (`u16` opcode with `0x8000` negation flag + typed operands).
