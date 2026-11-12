# Feature Matrix — GTA: San Andreas vs. GTASARUST

Tracks feature parity between the original *Grand Theft Auto: San Andreas* (PC v1.00) and GTASARUST.

| Category | Original Feature | GTASARUST Status | Crate / Module |
| --- | --- | --- | --- |
| **Assets & Formats** | IMG v2 (`gta3.img`, `gta_int.img`, `player.img`, `cutscene.img`) | Tested (`E:\gtasa`, 8 archives) | `gtasarust-gta-formats` |
| **Assets & Formats** | DFF RenderWare Clump / Geometry / FrameList / Materials | Tested (15,334 DFFs indexed; textured `DffSubMesh` + frame hierarchy rendered) | `gtasarust-gta-formats`, `gtasarust-renderer` |
| **Assets & Formats** | TXD RenderWare TextureDictionary (DXT1/3/5, RGBA, Palettized) | Tested (3,983 archives indexed; `565` opaque fix + `alpha_name` mask compositing + `txdp` chains; `3,008` mipmapped textures uploaded) | `gtasarust-gta-formats`, `gtasarust-renderer` |
| **Assets & Formats** | COL1 / COL2 / COL3 / COL4 collision archives | Tested (`10,160` models indexed & attached as active-sector `rapier3d` colliders) | `gtasarust-gta-formats`, `gtasarust-physics` |
| **Assets & Formats** | IDE object/vehicle/ped definitions & text/binary IPL placements | Tested (14,344 objects, `txdp` chains, `IDE_FLAG_*` routing, `50,935` placements, `1,045` `CARS`, `376` `ENEX`) | `gtasarust-gta-formats`, `gtasarust-world` |
| **Assets & Formats** | GXT localization tables (`american.gxt`, etc.) | Tested (16,588 entries parsed & queried) | `gtasarust-gta-formats`, `gtasarust-localization` |
| **Assets & Formats** | `handling.cfg`, `weapon.dat`, `timecyc.dat`, `water.dat`, `carcols.dat` | Tested (210 handling, 30 weapons, 184 timecyc, 301 water quads) | `gtasarust-gta-formats` |
| **Assets & Formats** | IFP (`ANPK` / `ANP3`) skeletal animations | Tested (`294` `ped.ifp` clips decoded w/ `KR00`/`KRT0` keyframes & linear-blend skinning) | `gtasarust-gta-formats`, `gtasarust-animation` |
| **Assets & Formats** | `main.scm` & `script.img` bytecode VM | In progress (`ScmImage` chunks + `699` SA opcodes + `ScmVm` multi-thread execution w/ `ScmWorldCommand` dispatch; campaign opcodes expanding) | `gtasarust-gta-formats`, `gtasarust-scripting` |
| **Engine & Graphics** | `wgpu` 3D renderer, perspective camera, directional sun, fog | Tested (Vulkan RX 480; 3-pass opaque/double-sided/blend pipelines, `ExtraVertColour` D/N, sRGB gamma fix, `.col` collided camera) | `gtasarust-renderer` |
| **Engine & Graphics** | Timecycle sky/clouds/day-night/weather transitions | Tested / In progress (`timecyc.dat` sky, sun, `ambient_obj`, water & `dn_balance` interpolation live in shaders) | `gtasarust-renderer`, `gtasarust-world` |
| **Engine & Graphics** | LOD system, frustum culling, GPU instancing, sector streaming | Tested (`bounding_radius` frustum culling + instancing + `is_lod` skyline outside detail radius + hour schedules) | `gtasarust-renderer`, `gtasarust-world` |
| **Engine & Graphics** | Water rendering & reflections | In progress (`301` `water.dat` quads uploaded and rendered in transparent pass w/ timecycle `water_color`; reflections/buoyancy pending) | `gtasarust-renderer`, `gtasarust-world` |
| **World** | Los Santos, San Fierro, Las Venturas, Countryside & Desert | In progress (placements & zones loaded across full map) | `gtasarust-world` |
| **World** | Road/ped path nodes (`NODES*.DAT`), interiors, railways | Tested / In progress (`68,237` nodes / `143,622` links, `376` `ENEX` portals, `gta_int.img` interior cells active; railways pending) | `gtasarust-gta-formats`, `gtasarust-world`, `gtasarust-ai` |
| **Player** | Third-person movement (walk/run/sprint/jump/crouch/swim/climb) | In progress (modular skinned CJ from `player.img` + `ped.ifp`, camera-relative movement/aim turn, decoupled mouse orbit + pointer lock) | `gtasarust-entities`, `gtasarust-animation` |
| **Vehicles** | Cars, bikes, BMX, boats, planes, helicopters, trains, emergency | In progress (`handling.cfg` physics, 4-wheel dummy mounts w/ steering & rolling spin, 4-point `.col` pitch/roll alignment) | `gtasarust-vehicles`, `gtasarust-renderer`, `gtasarust-physics` |
| **AI & Law** | Pedestrian AI, traffic AI, 6-star police & wanted system | In progress (`NODES*.DAT` skinned sidewalk peds `fam2`/`fam3`/`bmydj`/`lapd1`/`hmost`, traffic driving/yielding, carjacking panic, police pursuit) | `gtasarust-ai` |
| **Gameplay** | Health, armor, money, stamina, respect, weapons, shops, garages | In progress (health/armor, Colt .45 inventory/firing, Johnson House safehouse heal+save, Cluckin' Bell & Ammu-Nation interior counters) | `gtasarust-entities`, `gtasarust-weapons`, `gtasarust-game` |
| **Story** | Complete campaign missions, cutscenes, side activities | In progress (checkpoint campaign framework + `main.scm` boot/opcode dispatch) | `gtasarust-missions`, `gtasarust-scripting` |
| **Audio** | SFX banks, vehicle engines, footsteps, radio stations, volume mixer | In progress (`audio/streams` XOR Ogg Vorbis radio streaming & `audio/SFX` PCM footsteps/gunshots/horn/crash active via `kira`; engine RPM pitch synthesis pending) | `gtasarust-audio`, `gtasarust-gta-formats` |
| **UI & Saves** | Authentic SA HUD, radar, menus, 8 versioned save slots | Tested / In progress (`LOADSCS.TXD` splash, `hud.txd` sprites, `144` `radar00..143` circular minimap & 12x12 fullscreen map + SHA-256 saves) | `gtasarust-ui`, `gtasarust-save-system` |
