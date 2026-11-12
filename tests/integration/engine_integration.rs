use glam::Vec3;
use gtasarust_engine::{EngineConfig, FixedTimestep, GameClock, HostInfo};
use gtasarust_physics::PhysicsWorld;
use gtasarust_save_system::{SaveGameData, read_save_slot, write_save_slot};
use gtasarust_world::WorldManager;

#[test]
fn engine_world_physics_and_save_integrate_end_to_end() {
    let host = HostInfo::detect();
    assert_eq!(host.arch, "x86_64");

    let cfg = EngineConfig::default();
    assert!(cfg.graphics.draw_distance >= 200.0);

    let mut world = WorldManager::create_fallback_san_andreas();
    let grove = Vec3::new(2495.3, -1688.2, 1.0);
    world.update_streaming(grove, cfg.graphics.draw_distance);
    assert!(!world.active_sectors.is_empty());

    let mut ts = FixedTimestep::default();
    let mut clock = GameClock::default();
    let mut physics = PhysicsWorld::new();
    let steps = ts.accumulate(0.051);
    assert_eq!(steps, 3);
    for _ in 0..steps {
        clock.advance(ts.fixed_dt);
        physics.step(ts.fixed_dt);
    }

    let tmp = std::env::temp_dir().join("gtasarust_test_int");
    let save = SaveGameData::default();
    write_save_slot(&tmp, 2, &save).unwrap();
    let loaded = read_save_slot(&tmp, 2).unwrap();
    assert_eq!(loaded.slot_title, "Grove Street - Home");
    let _ = std::fs::remove_dir_all(&tmp);
}
