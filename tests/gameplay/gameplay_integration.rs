use glam::{Vec2, Vec3};
use gtasarust_ai::{AiSimulation, CrimeEvent};
use gtasarust_entities::PlayerEntity;
use gtasarust_input::{ActionFrameState, ControlMode};
use gtasarust_missions::MissionManager;
use gtasarust_physics::PhysicsWorld;
use gtasarust_scripting::ScmVm;
use gtasarust_vehicles::{VehicleClass, VehicleManager, VehicleSpawnSpec};
use gtasarust_weapons::WeaponInventory;

#[test]
fn player_weapons_vehicles_police_and_missions_interact() {
    let mut physics = PhysicsWorld::new();
    let mut player = PlayerEntity::default();
    player
        .weapons
        .give_weapon(WeaponInventory::preset_colt45(), 34);
    player.weapons.cycle_next();

    let mut vehicles = VehicleManager::default();
    let car_id = vehicles.spawn(VehicleSpawnSpec {
        model_id: 492,
        name: "GREENWOO",
        class: VehicleClass::Car,
        position: player.position + Vec3::new(2.0, 0.0, 0.0),
        yaw: 0.0,
        color1: [0.1, 0.5, 0.2],
        handling: None,
    });

    let mut ai = AiSimulation::default();
    let mut vm = ScmVm::default();
    let mut missions = MissionManager::new_san_andreas_campaign();

    // Fire handgun -> builds partial wanted heat, repeated discharge triggers 1+ stars
    assert!(player.weapons.try_fire().is_some());
    ai.report_crime(CrimeEvent::WeaponDischarged, &mut player);
    assert_eq!(ai.wanted_stars, 0);
    ai.report_crime(CrimeEvent::WeaponDischarged, &mut player);
    assert!(ai.wanted_stars >= 1);

    // Enter vehicle -> switches control mode
    assert!(vehicles.toggle_player_entry(&mut player));
    assert_eq!(player.control_mode, ControlMode::InVehicle);
    assert_eq!(player.current_vehicle, Some(car_id));

    // Drive forward in vehicle & step physics/AI/missions
    let drive = ActionFrameState {
        move_axis: Vec2::new(0.0, 1.0),
        ..Default::default()
    };
    for _ in 0..30 {
        let v = vehicles.get_mut(car_id).unwrap();
        v.step_physics(1.0 / 60.0, Some(&drive), &physics);
        player.position = v.position;
        physics.step(1.0 / 60.0);
        ai.tick(1.0 / 60.0, &mut player, &mut vehicles);
        missions.tick(1.0 / 60.0, &mut player, &mut vm);
    }

    assert!(vehicles.get(car_id).unwrap().speed_mps() > 2.0);
}
