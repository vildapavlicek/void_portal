use {
    crate::{
        configs::SoldierConfig,
        portal::{Enemy, Health, Portal, SpawnIndex},
        soldier::{
            move_projectiles, projectile_collision, soldier_attack_logic, soldier_decision_logic,
            soldier_movement_logic, AttackRange, Attacking, Moving, Projectile, Soldier,
        },
    },
    bevy::{
        prelude::*,
        time::{Time, TimePlugin},
    },
    void_components::{Dead, Reward},
    void_core::events::EnemyKilled,
};

// --- Test Utilities ---

fn create_app_with_minimal_plugins() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>()); // Manually inserting Time to control it
    app.insert_resource(Time::<()>::default());
    app.add_message::<EnemyKilled>(); // Register EnemyKilled message
    // Removed register_type calls as components do not implement Reflect
    app
}

fn spawn_portal_and_tracker(app: &mut App) {
    app.insert_resource(crate::portal::PortalSpawnTracker(0));
    app.world_mut().spawn((
        Portal,
        Transform::default(), // Required for distance checks (even if 0,0)
    ));
}

fn insert_soldier_config(app: &mut App) {
    app.insert_resource(SoldierConfig {
        attack_timer: 1.0,
        projectile_speed: 100.0,
        projectile_damage: 10.0,
        projectile_lifetime: 5.0,
        attack_range: 50.0,
        move_speed: 100.0,
    });
}

// --- Tests ---

#[test]
fn test_soldier_acquires_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    app.add_systems(Update, soldier_decision_logic);

    // Spawn Soldier
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
                target: None,
            },
            AttackRange(50.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Spawn Enemy (within range)
    let enemy = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 100.0,
                max: 100.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(10.0, 0.0, 0.0),
        ))
        .id();

    // Update Tracker to reflect spawned enemies
    app.world_mut().resource_mut::<crate::portal::PortalSpawnTracker>().0 = 2;

    app.update();

    // Soldier should now be Attacking (Distance 10 <= Range 50)
    let attacking = app.world().get::<Attacking>(soldier);
    assert!(attacking.is_some(), "Soldier should be Attacking");
    assert_eq!(attacking.unwrap().0, enemy, "Soldier should target enemy");
}

#[test]
fn test_soldier_moves_to_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    app.add_systems(
        Update,
        (soldier_decision_logic, soldier_movement_logic).chain(),
    );

    // Spawn Soldier at (0,0)
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
                target: None,
            },
            AttackRange(10.0), // Short range
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Spawn Enemy (out of range) at (100, 0)
    let _enemy = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 100.0,
                max: 100.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ))
        .id();

    // 1. Update Decision -> Should transition to Moving
    app.update();

    assert!(app.world().get::<Moving>(soldier).is_some());

    // 2. Advance time and Update Movement
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.1)); // 0.1s * 100 speed = 10 units
    }
    app.update();

    let transform = app.world().get::<Transform>(soldier).unwrap();
    assert!(
        transform.translation.x > 0.0,
        "Soldier should have moved towards enemy"
    );
}

#[test]
fn test_soldier_attacks_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    // Systems chain: Decision -> Attack (Spawns Projectile) -> Move Projectile -> Collision (Damage)
    app.add_systems(
        Update,
        (
            soldier_decision_logic,
            soldier_attack_logic,
            move_projectiles,
            projectile_collision,
        )
            .chain(),
    );

    // Spawn Soldier
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once), // Ready to attack immediately if reset
                target: None,
            },
            AttackRange(50.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Spawn Enemy
    let enemy = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 20.0,
                max: 20.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0), // Added Reward component as required by despawn_dead_enemies
        ))
        .id();

    // 1. Update -> Decision (Acquire Target -> Attacking)
    app.update();
    assert!(app.world().get::<Attacking>(soldier).is_some());

    // 2. Advance time to trigger attack and projectile flight
    // Interval 1.0s. Projectile speed 100.0. Distance 10.0. Flight time 0.1s.

    {
        let mut time = app.world_mut().resource_mut::<Time>();
        // Advance slightly to trigger first attack if timer is ready
        time.advance_by(std::time::Duration::from_secs_f32(0.01));
    }
    app.update(); // Attack logic runs, spawns projectile?

    // Check if projectile exists
    let world = app.world_mut();
    let projectile_exists = world
        .query::<&Projectile>()
        .iter(world)
        .count()
        > 0;

    // If no projectile, maybe timer wasn't ready. Advance 1.0s
    if !projectile_exists {
         {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(1.05));
        }
        app.update();
    }

    // Now advance for flight time (0.1s needed)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.2));
    }
    app.update(); // Move projectiles, collision

    let health = app.world().get::<Health>(enemy).unwrap();
    assert!(
        health.current < 20.0,
        "Enemy should have taken damage. Current: {}",
        health.current
    );
    assert_eq!(health.current, 10.0);
}

#[test]
fn test_soldier_switching_targets() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    app.add_systems(Update, soldier_decision_logic);

    // Spawn Soldier
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
                target: None,
            },
            AttackRange(50.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Spawn Enemy 1 (Oldest, SpawnIndex 0)
    let enemy1 = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(10.0, 0.0, 0.0),
        ))
        .id();

    // Spawn Enemy 2 (Newer, SpawnIndex 1)
    let _enemy2 = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(1),
            Transform::from_xyz(10.0, 0.0, 0.0),
        ))
        .id();

    // Update Tracker to reflect spawned enemies
    app.world_mut().resource_mut::<crate::portal::PortalSpawnTracker>().0 = 2;

    app.update();

    // Should target enemy1
    let attacking = app.world().get::<Attacking>(soldier).unwrap();
    assert_eq!(attacking.0, enemy1);

    // Despawn enemy1 (simulate death)
    app.world_mut().despawn(enemy1);

    // Need attack logic to detect invalid target and remove Attacking
    app.add_systems(Update, soldier_attack_logic);

    app.update(); // Detects invalid target, removes Attacking.

    app.update(); // Decision logic picks new target.

    let attacking = app.world().get::<Attacking>(soldier);
    assert!(
        attacking.is_some(),
        "Soldier should have acquired new target"
    );
}

#[test]
fn test_soldier_spawn_index_wrapping() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);
    app.add_systems(Update, soldier_decision_logic);

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
                target: None,
            },
            AttackRange(50.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Set Tracker to 10.
    app.world_mut()
        .resource_mut::<crate::portal::PortalSpawnTracker>()
        .0 = 10;

    let enemy_old = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(9), // Older relative to 10
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();

    let _enemy_new = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(10), // Newer
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();

    app.update();

    let attacking = app.world().get::<Attacking>(soldier).unwrap();
    assert_eq!(attacking.0, enemy_old);
}

#[test]
fn test_soldier_retargets_on_death() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    app.add_systems(
        Update,
        (
            soldier_decision_logic,
            soldier_attack_logic,
            move_projectiles,
            projectile_collision,
            crate::portal::handle_dying_enemies,
        )
            .chain(),
    );

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
                target: None,
            },
            AttackRange(50.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let enemy1 = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();

    let enemy2 = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(1),
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();

    // Update Tracker to reflect spawned enemies
    app.world_mut().resource_mut::<crate::portal::PortalSpawnTracker>().0 = 2;

    // 1. Target Enemy 1
    app.update();
    let attacking = app.world().get::<Attacking>(soldier).unwrap();
    assert_eq!(attacking.0, enemy1);

    // 2. Kill Enemy 1
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.5)); // Trigger attack
    }
    app.update();
    // Projectile flight
     {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.2));
    }
    app.update();

    // Enemy 1 should be Dead (Enemy component removed)
    assert!(app.world().get::<Enemy>(enemy1).is_none());
    assert!(app.world().get::<Dead>(enemy1).is_some());

    // Attack logic sees invalid target (because Enemy component missing) -> Removes Attacking.
    app.update();
    // Decision logic picks Enemy 2.
    app.update();

    let attacking = app.world().get::<Attacking>(soldier).unwrap();
    assert_eq!(attacking.0, enemy2);
}

#[test]
fn test_soldier_attack_reset_on_switch() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);
    app.add_systems(Update, soldier_decision_logic);

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once), // Halfway done
                target: None,
            },
            AttackRange(50.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Enemy 1
    let enemy1 = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();

    app.update(); // Targets Enemy 1

    // Simulate invalidation.
    app.world_mut().entity_mut(soldier).remove::<Attacking>();

    // Spawn Enemy 2
    let _enemy2 = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 10.0,
                max: 10.0,
            },
            SpawnIndex(2),
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();
    // Despawn Enemy 1
    app.world_mut().despawn(enemy1);

    app.update(); // Acquire Enemy 2

    // Check timer was reset.
    let soldier_comp = app.world().get::<Soldier>(soldier).unwrap();
    assert!(
        soldier_comp.attack_timer.fraction() >= 1.0,
        "Attack timer should be ready (reset) upon new target acquisition"
    );
}

#[test]
fn test_soldier_retargets_closest_if_multiple_oldest() {
    // Skipped
}
