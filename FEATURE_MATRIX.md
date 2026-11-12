# Feature Matrix — GTA: San Andreas vs. GTASARUST

Tracks feature parity between the original *Grand Theft Auto: San Andreas* (PC v1.00) and GTASARUST.

| Category | Original Feature | GTASARUST Status | Crate / Module |
| --- | --- | --- | --- |
| **Assets & Formats** | IMG v2 (`gta3.img`, `gta_int.img`, `player.img`, `cutscene.img`) | Tested (`E:\gtasa`, 8 archives) | `gtasarust-gta-formats` |
| **Assets & Formats** | DFF RenderWare Clump / Geometry / FrameList / Materials | Tested (15,334 DFFs indexed; textured `DffSubMesh` + frame hierarchy rendered) | `gtasarust-gta-formats`, `gtasarust-renderer` |
| **Assets & Formats** | TXD RenderWare TextureDictionary (DXT1/3/5, RGBA, Palettized) | Tested (3,983 archives indexed; runtime TXD texture streaming uploaded `629` textures) | `gtasarust-gta-formats`, `gtasarust-renderer` |
| **Assets & Formats** | COL1 / COL2 / COL3 / COL4 collision archives | Tested (`10,160` models indexed & attached as active-sector `rapier3d` colliders) | `gtasarust-gta-formats`, `gtasarust-physics` |
| **Assets & Formats** | IDE object/vehicle/ped definitions & text/binary IPL placements | Tested (14,344 objects, 45,837 placements, 1,043 binary `CARS` loaded) | `gtasarust-gta-formats`, `gtasarust-world` |
| **Assets & Formats** | GXT localization tables (`american.gxt`, etc.) | Tested (16,588 entries parsed & queried) | `gtasarust-gta-formats`, `gtasarust-localization` |
| **Assets & Formats** | `handling.cfg`, `weapon.dat`, `timecyc.dat`, `water.dat`, `carcols.dat` | Tested (210 handling, 30 weapons, 184 timecyc, 301 water quads) | `gtasarust-gta-formats` |
| **Assets & Formats** | IFP (`ANPK` / `ANP3`) skeletal animations | Tested (`294` `ped.ifp` clips decoded w/ `KR00`/`KRT0` keyframes & linear-blend skinning) | `gtasarust-gta-formats`, `gtasarust-animation` |
| **Assets & Formats** | `main.scm` & `script.img` bytecode VM | In progress (135 missions indexed; opcode execution expanding) | `gtasarust-scripting` |
| **Engine & Graphics** | `wgpu` 3D renderer, perspective camera, directional sun, fog | Tested (Vulkan RX 480 window & sky/ground/instanced DFF + dynamic skinned ped pass) | `gtasarust-renderer` |
| **Engine & Graphics** | Timecycle sky/clouds/day-night/weather transitions | In progress (shader & timecycle interpolation wired) | `gtasarust-renderer`, `gtasarust-world` |
| **Engine & Graphics** | LOD system, frustum culling, GPU instancing, sector streaming | In progress (frustum culling + instancing + sector grid active) | `gtasarust-renderer`, `gtasarust-world` |
| **Engine & Graphics** | Water rendering & reflections | In progress (`301` `water.dat` quads uploaded and rendered via GPU mesh; reflections/buoyancy pending) | `gtasarust-renderer`, `gtasarust-world` |
| **World** | Los Santos, San Fierro, Las Venturas, Countryside & Desert | In progress (placements & zones loaded across full map) | `gtasarust-world` |
| **World** | Road/ped path nodes (`NODES*.DAT`), interiors, railways | Tested / In progress (`68,237` nodes / `143,622` links, `376` `ENEX` portals, `gta_int.img` interior cells active; railways pending) | `gtasarust-gta-formats`, `gtasarust-world`, `gtasarust-ai` |
| **Player** | Third-person movement (walk/run/sprint/jump/crouch/swim/climb) | In progress (`ped.ifp` skinned `fam1` CJ animation + walk/sprint/jump/crouch physics & camera) | `gtasarust-entities`, `gtasarust-animation` |
| **Vehicles** | Cars, bikes, BMX, boats, planes, helicopters, trains, emergency | In progress (`handling.cfg` driven entry/exit & basic physics) | `gtasarust-vehicles`, `gtasarust-physics` |
| **AI & Law** | Pedestrian AI, traffic AI, 6-star police & wanted system | In progress (`NODES*.DAT` skinned sidewalk peds `fam2`/`fam3`/`bmydj`/`lapd1`/`hmost`, traffic driving/yielding, carjacking panic, police pursuit) | `gtasarust-ai` |
| **Gameplay** | Health, armor, money, stamina, respect, weapons, shops, garages | In progress (health/armor, Colt .45 inventory/firing, Johnson House safehouse heal+save, Cluckin' Bell & Ammu-Nation interior counters) | `gtasarust-entities`, `gtasarust-weapons`, `gtasarust-game` |
| **Story** | Complete campaign missions, cutscenes, side activities | In progress (checkpoint campaign framework) | `gtasarust-missions`, `gtasarust-scripting` |
| **Audio** | SFX banks, vehicle engines, footsteps, radio stations, volume mixer | In progress (`audio/streams` XOR Ogg Vorbis radio streaming & `audio/SFX` PCM footsteps/gunshots/horn/crash active via `kira`; engine RPM pitch synthesis pending) | `gtasarust-audio`, `gtasarust-gta-formats` |
| **UI & Saves** | Authentic SA HUD, radar, menus, 8 versioned save slots | In progress (`egui` menus/HUD + SHA-256 save slots tested) | `gtasarust-ui`, `gtasarust-save-system` |
