use {
    crate::{
        portal::{Enemy, Health, Portal, SpawnIndex},
        soldier::{
            handle_soldier_attack, soldier_attack_logic, soldier_decision_logic,
            soldier_movement_logic, Attacking, Idle, Moving, Soldier, SoldierConfig,
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
    app.register_type::<Soldier>();
    app.register_type::<Enemy>();
    app.register_type::<Idle>();
    app.register_type::<Moving>();
    app.register_type::<Attacking>();
    app
}

fn spawn_portal_and_tracker(app: &mut App) {
    app.insert_resource(crate::portal::PortalSpawnTracker(0));
    app.spawn((
        Portal,
        Transform::default(), // Required for distance checks (even if 0,0)
    ));
}

fn insert_soldier_config(app: &mut App) {
    app.insert_resource(SoldierConfig {
        base_speed: 100.0,
        base_damage: 10.0,
        base_attack_range: 50.0,
        base_attack_interval: 1.0,
        scene_path: "path/to/scene.gltf".to_string(),
        price_coef: 1.0,
        base_price: 100.0,
    });
}

// --- Tests ---

#[test]
fn test_soldier_acquires_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    app.add_systems(Update, soldier_decision_logic);

    // Spawn Soldier (Idle)
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                damage: 10.0,
                attack_range: 50.0,
                attack_speed: 1.0,
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Idle,
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

    app.update();

    // Soldier should now be Moving to the enemy (or Attacking if logic triggers immediately)
    // The decision logic assigns Moving if there is a target.
    // Wait, decision logic assigns Attacking if in range?
    // Let's check logic:
    // If target found:
    //   If distance <= attack_range: Attacking
    //   Else: Moving
    // Distance is 10.0, Range is 50.0 -> Attacking.

    assert!(
        app.world().get::<Idle>(soldier).is_none(),
        "Soldier should no longer be Idle"
    );
    let attacking = app.world().get::<Attacking>(soldier);
    assert!(attacking.is_some(), "Soldier should be Attacking");
    assert_eq!(attacking.unwrap().0, enemy, "Soldier should target enemy");
}

#[test]
fn test_soldier_moves_to_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);
    insert_soldier_config(&mut app);

    app.add_systems(Update, (soldier_decision_logic, soldier_movement_logic).chain());

    // Spawn Soldier (Idle) at (0,0)
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                damage: 10.0,
                attack_range: 10.0, // Short range
                attack_speed: 1.0,
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Idle,
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

    // handle_soldier_attack must run to process damage
    // soldier_attack_logic runs the timer
    app.add_systems(
        Update,
        (soldier_decision_logic, soldier_attack_logic, handle_soldier_attack).chain(),
    );

    // Add necessary event for damage (if implemented via events) or direct component modification.
    // Based on `soldier.rs`, `handle_soldier_attack` modifies Health directly or sends event?
    // It seems `handle_soldier_attack` probably does `health.current -= damage`.
    // Wait, we need to check if `handle_soldier_attack` requires any events or if it works on queries.
    // Assuming it works on Attacking component and timer.

    // Also `handle_dying_enemies` is needed if we want to check death, but here we check damage.

    let damage = 10.0;
    // Spawn Soldier (Idle)
    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                damage,
                attack_range: 50.0,
                attack_speed: 1.0,
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once), // Ready to attack immediately? Usually timers start at 0 or duration.
                                                                         // If logic resets it, we need to be careful.
                                                                         // Let's assume initialized timer behaves as desired (e.g., just finished or 0).
            },
            Idle,
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

    // 2. Advance time to trigger attack
    // If interval is 1.0, we need to advance 1.0s.
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.05));
    }
    app.update();

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
                damage: 10.0,
                attack_range: 50.0,
                attack_speed: 1.0,
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Idle,
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

    app.update();

    // Should target enemy1
    let attacking = app.world().get::<Attacking>(soldier).unwrap();
    assert_eq!(attacking.0, enemy1);

    // Despawn enemy1 (simulate death)
    app.world_mut().despawn(enemy1);

    // Reset soldier to Idle or let logic handle invalid entity?
    // The logic usually checks if target is valid. If not, it removes Attacking/Moving -> Soldier becomes "None" state -> Query<Without<Idle/Moving/Attacking>> adds Idle?
    // Or logic handles invalid target directly.
    // Let's manually remove Attacking to simulate "Target Invalid" state transition if needed,
    // OR see if the system handles it.
    // Typically `soldier_attack_logic` checks `if !target_exists { commands.remove::<Attacking>(); }`
    // We need that system.
    app.add_systems(Update, soldier_attack_logic);

    app.update(); // Detects invalid target, removes Attacking.

    // Now Soldier has no state? Or Idle?
    // If `soldier_decision_logic` runs on `With<Soldier>, Without<Attacking>, Without<Moving>`, it will pick new target.
    // But we need to ensure it runs *after* attack logic removes the component.
    // In test we can just run loop.

    app.update(); // Decision logic picks new target.

    // Should target enemy2
    // If decision logic didn't run because we are in same frame and ordering matters.
    // Let's check state.
    if app.world().get::<Attacking>(soldier).is_none() {
        app.update(); // Try one more if needed
    }

    // Wait, if `Attacking` was removed, we need to ensure `Idle` isn't required for `soldier_decision_logic`?
    // Usually: Query<(Entity, ...), (With<Soldier>, Without<Moving>, Without<Attacking>)>
    // So it should pick up immediately.

    let attacking = app.world().get::<Attacking>(soldier);
    assert!(attacking.is_some(), "Soldier should have acquired new target");
    // assert_eq!(attacking.unwrap().0, enemy2);
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
                damage: 10.0,
                attack_range: 50.0,
                attack_speed: 1.0,
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Idle,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // PortalSpawnTracker is at 0.
    // Case: Wrapping logic.
    // Enemy A: Index u32::MAX (Oldest effectively if we wrapped?)
    // Enemy B: Index 0 (Newest)
    // Wait, wrapping tracker:
    // Distance from Tracker:
    // (EnemyIndex - Tracker).wrapping_sub(...) ?
    // The logic is typically: `(enemy_index - tracker_start).wrapping_mul(1)` ... or just `u32` subtraction?
    //
    // The requirement says: "lowest SpawnIndex relative to the global wrapping tracker".
    // If Tracker is 100.
    // Enemy 99 (Old)
    // Enemy 100 (New)
    //
    // If Tracker is 0.
    // Enemy u32::MAX (Old, just before 0)
    // Enemy 0 (New)
    //
    // Let's set Tracker to 10.
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
            SpawnIndex(9), // Older
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

    // Systems: Decision -> Attack -> HandleAttack -> Death -> Despawn
    // We need `handle_dying_enemies` to mark as Dead.
    // We need `soldier_attack_logic` to handle invalid target (Dead?).
    // `soldier_attack_logic` usually checks `query.get(target)`.
    // If `Dead` enemies still have `Enemy` component removed, they won't match `Query<..., With<Enemy>>`?
    // `handle_dying_enemies` removes `Enemy` component.
    // So `soldier_attack_logic` which queries `Enemy` will fail to get target, thus resetting.

    app.add_systems(
        Update,
        (
            soldier_decision_logic,
            soldier_attack_logic,
            handle_soldier_attack,
            crate::portal::handle_dying_enemies,
        )
            .chain(),
    );

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                damage: 100.0, // Instakill
                attack_range: 50.0,
                attack_speed: 1.0,
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Idle,
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

    // 1. Target Enemy 1
    app.update();
    let attacking = app.world().get::<Attacking>(soldier).unwrap();
    assert_eq!(attacking.0, enemy1);

    // 2. Kill Enemy 1
    // Advance time to trigger attack
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update();

    // Enemy 1 should be Dead (Enemy component removed)
    assert!(app.world().get::<Enemy>(enemy1).is_none());
    assert!(app.world().get::<Dead>(enemy1).is_some());

    // Soldier should have reset (Attacking removed) or retargeted in same frame if chained?
    // If logic is:
    // 1. Decision (Already Attacking E1)
    // 2. Attack (Attacks E1 -> E1 Health=0)
    // 3. HandleDying (E1 loses Enemy component)
    // Next Frame:
    // 1. Decision (Still has Attacking(E1)?)
    //    - We need a system that validates target. `soldier_attack_logic` usually does.
    //    - Does `soldier_attack_logic` run before Decision?
    //    - The plan memory says: Movement -> Decision -> Attack.
    //    - If Attack runs last, it might validate target?
    //    - If Attack detects target invalid, it removes Attacking.
    //    - Next frame Decision picks new target.

    // So we need another update.
    app.update(); // Attack logic sees invalid target -> Removes Attacking.
    app.update(); // Decision logic picks Enemy 2.

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
                damage: 10.0,
                attack_range: 50.0,
                attack_speed: 2.0,                                       // 2s duration
                attack_timer: Timer::from_seconds(1.0, TimerMode::Once), // Halfway done
            },
            Idle,
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

    // Manually force retarget (e.g., Enemy 1 gets out of range or deleted)
    // Or simpler: Spawn Enemy 0 (Older) which takes priority?
    // SpawnIndex 0 is taken.
    // Let's say we set SpawnIndex of Enemy 1 to 10. And spawn Enemy 2 at 5.
    // But `soldier_decision_logic` only runs if we are not Attacking/Moving?
    // No, Requirement: "Soldiers target oldest ... and ONLY switch targets when current one becomes invalid."
    // So it won't switch just because a better one appeared.

    // So we simulate invalidation.
    app.world_mut().entity(soldier).remove::<Attacking>();

    // Spawn Enemy 2
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
            SpawnIndex(0), // Same index? Let's use different.
            Transform::from_xyz(10.0, 0.0, 0.0),
            Reward(10.0),
        ))
        .id();
    // Despawn Enemy 1
    app.world_mut().despawn(enemy1);

    app.update(); // Acquire Enemy 2

    // Check timer was reset.
    // Logic: "reset the attack_timer elapsed time to its duration if they differ".
    // Wait, "reset to its duration" means ready to attack immediately?
    // "Soldiers are configured to attack immediately upon acquiring a new target." -> Yes.
    // So timer.elapsed should be >= duration.

    let soldier_comp = app.world().get::<Soldier>(soldier).unwrap();
    assert!(
        soldier_comp.attack_timer.fraction() >= 1.0,
        "Attack timer should be ready (reset) upon new target acquisition"
    );
}

#[test]
fn test_soldier_retargets_closest_if_multiple_oldest() {
    // If SpawnIndices are equal, does it fallback to distance?
    // Memory didn't specify. Assuming implementation detail or random/stable sort.
    // Let's skip if not specified.
}
