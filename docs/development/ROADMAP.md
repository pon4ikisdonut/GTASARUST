# GTASARUST Development Roadmap

See [DEVELOPMENT_STATUS.md](../../DEVELOPMENT_STATUS.md) for current live status.

1. **Phase 1 — Foundation**: GitHub repo, Cargo workspace, platform detection, case-insensitive VFS, logging, TOML config, application lifecycle, Windows/Linux x64 CI.
2. **Phase 2 — Engine**: `winit` windowing, `wgpu` 3D pipeline, perspective/third-person camera, `rapier3d` physics world, `kira` audio mixer, `gilrs` + keyboard/mouse input.
3. **Phase 3 — Original Asset Pipeline**: Clean-room parsers (`VER2` IMG, RenderWare DFF/TXD, COL1-4, IDE, text & binary IPL, GXT, DAT, handling/weapon/timecyc/water/carcols, IFP, SCM) + batch extractor, converter, manifest & validator tools.
4. **Phase 4 — World**: Full San Andreas sector streaming, terrain & building meshes, TXD texture binding, COL static geometry, water polygons, occlusion/LODs.
5. **Phase 5 — Player**: CJ skeletal mesh, IFP animation state machine, camera-relative movement, melee, aiming, shooting, swimming, climbing.
6. **Phase 6 — Vehicles**: All vehicle classes using `handling.cfg`, suspension, tire friction, deformation/damage, lights, entry/exit, traffic.
7. **Phase 7 — World Simulation**: Pedestrians, traffic path graph (`NODES*.DAT`), 6-star police AI, weather, 24h timecycle.
8. **Phase 8 — Gameplay**: Weapon inventory, shops, clothing/hair/tattoos, safehouses, garages, gang territories, collectibles, stats.
9. **Phase 9 — Story**: SCM VM opcode execution, mission scripts, cutscenes, voice/dialogue, rewards, progression.
10. **Phase 10 — Final Game**: Authentic menus, HUD, map screen, 8-slot versioned save system, full audio/radio integration.
11. **Phase 11 — Compatibility**: Windows & Linux x64 driver/performance/memory/crash testing.
12. **Phase 12 — Release**: Release binaries, user asset setup guide, documentation.
