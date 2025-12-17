use {
    crate::*,
    bevy::time::{Time, TimePlugin},
    common::{events::DamageMessage, MonsterKilled},
    items::{AttackRange as ItemAttackRange, BaseDamage, Melee, ProjectileStats, Ranged},
    monsters::{apply_damage_logic, Health, Monster, SpawnIndex},
    portal::PortalSpawnTracker,
};

// --- Test Utilities ---

fn create_app_with_minimal_plugins() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>()); // Manually inserting Time to control it
    app.insert_resource(Time::<()>::default());
    app.add_message::<MonsterKilled>(); // Register MonsterKilled message
    app.add_message::<DamageMessage>(); // Register DamageMessage

    // Register types
    app.register_type::<PlayerNpc>()
        .register_type::<MovementSpeed>()
        .register_type::<Target>()
        .register_type::<Weapon>()
        .register_type::<WeaponCooldown>()
        .register_type::<Projectile>()
        .register_type::<ItemAttackRange>()
        .register_type::<BaseDamage>()
        .register_type::<Melee>()
        .register_type::<Ranged>()
        .register_type::<ProjectileStats>();

    app
}

fn spawn_portal_and_tracker(app: &mut App) {
    app.insert_resource(PortalSpawnTracker(0));
    app.world_mut().spawn((Transform::default(),));
}

#[test]
fn test_npc_acquires_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);

    app.add_systems(Update, player_npc_decision_logic);

    // Spawn PlayerNpc
    let npc = app
        .world_mut()
        .spawn((PlayerNpc, Target(None), Transform::default(), Intent::Idle))
        .id();

    // Spawn Monster
    let monster = app
        .world_mut()
        .spawn((
            Monster {
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

    // Update Tracker
    app.world_mut().resource_mut::<PortalSpawnTracker>().0 = 1;

    app.update();

    let target = app.world().get::<Target>(npc).unwrap();
    assert_eq!(target.0, Some(monster), "NPC should target the monster");
}

#[test]
fn test_npc_moves_to_target() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);

    app.add_systems(
        Update,
        (player_npc_decision_logic, player_npc_movement_logic).chain(),
    );

    // Spawn NPC
    let npc = app
        .world_mut()
        .spawn((
            PlayerNpc,
            Target(None),
            MovementSpeed(100.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Intent::Idle,
        ))
        .id();

    // Spawn Weapon Child (Effective Range 20.0)
    let child = app.world_mut().spawn((Weapon, ItemAttackRange(20.0))).id();
    app.world_mut().entity_mut(npc).add_child(child);

    // Spawn Monster (Distance 100.0)
    let _monster = app
        .world_mut()
        .spawn((
            Monster {
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

    // Update Tracker
    app.world_mut().resource_mut::<PortalSpawnTracker>().0 = 1;

    // 1. Decision Logic
    app.update();

    // 2. Advance Time & Move
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.1)); // 10 units
    }
    app.update();

    let transform = app.world().get::<Transform>(npc).unwrap();
    assert!(
        transform.translation.x > 0.0,
        "NPC should move towards target"
    );
    assert!(
        transform.translation.x < 100.0,
        "NPC should not overshot instantly"
    );
}

#[test]
fn test_npc_stops_in_range() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);

    app.add_systems(
        Update,
        (player_npc_decision_logic, player_npc_movement_logic).chain(),
    );

    // Spawn NPC at 80.0
    let npc = app
        .world_mut()
        .spawn((
            PlayerNpc,
            Target(None),
            MovementSpeed(100.0),
            Transform::from_xyz(80.0, 0.0, 0.0),
            Intent::Idle,
        ))
        .id();

    // Spawn Weapon Child (Effective Range 30.0)
    let child = app.world_mut().spawn((Weapon, ItemAttackRange(30.0))).id();
    app.world_mut().entity_mut(npc).add_child(child);

    // Spawn Monster at 100.0. Distance = 20.0. Range = 30.0. Should NOT move.
    app.world_mut().spawn((
        Monster {
            target_position: Vec2::ZERO,
        },
        Health {
            current: 100.0,
            max: 100.0,
        },
        SpawnIndex(0),
        Transform::from_xyz(100.0, 0.0, 0.0),
    ));

    app.world_mut().resource_mut::<PortalSpawnTracker>().0 = 1;

    app.update(); // Decision
    app.update(); // Movement

    let transform = app.world().get::<Transform>(npc).unwrap();
    assert_eq!(
        transform.translation.x, 80.0,
        "NPC should not move if in range"
    );
}

#[test]
fn test_melee_attack() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);

    app.add_systems(
        Update,
        (
            player_npc_decision_logic,
            melee_attack_emit,
            apply_damage_logic,
        )
            .chain(),
    );

    // Spawn NPC
    let npc = app
        .world_mut()
        .spawn((
            PlayerNpc,
            Target(None),
            Transform::from_xyz(90.0, 0.0, 0.0),
            Intent::Idle,
        ))
        .id();

    // Spawn Melee Weapon Child
    let child = app
        .world_mut()
        .spawn((
            Weapon,
            Melee,
            BaseDamage(10.0),
            ItemAttackRange(20.0),
            WeaponCooldown {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
        ))
        .id();
    app.world_mut().entity_mut(npc).add_child(child);

    // Spawn Monster at 100.0. Distance 10.0 <= Range 20.0.
    let monster = app
        .world_mut()
        .spawn((
            Monster {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 50.0,
                max: 50.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ))
        .id();

    app.world_mut().resource_mut::<PortalSpawnTracker>().0 = 1;

    // 1. Decision (Acquire Target)
    app.update();

    // 2. Advance time to finish cooldown (1.0s)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        // Advance slightly more than 1.0 to ensure finished
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update(); // Attack logic

    let health = app.world().get::<Health>(monster).unwrap();
    assert_eq!(health.current, 40.0, "Monster should take 10 damage");
}

#[test]
fn test_ranged_attack_spawns_projectile() {
    let mut app = create_app_with_minimal_plugins();
    spawn_portal_and_tracker(&mut app);

    app.add_systems(
        Update,
        (
            player_npc_decision_logic,
            ranged_attack_logic,
            move_projectiles,
            projectile_collision,
            apply_damage_logic,
        )
            .chain(),
    );

    // Spawn NPC
    let npc = app
        .world_mut()
        .spawn((
            PlayerNpc,
            Target(None),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Intent::Idle,
        ))
        .id();

    // Spawn Ranged Weapon Child
    let child = app
        .world_mut()
        .spawn((
            Weapon,
            Ranged,
            BaseDamage(10.0),
            ItemAttackRange(200.0),
            WeaponCooldown {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
            ProjectileStats {
                speed: 100.0,
                lifetime: 5.0,
            },
        ))
        .id();
    app.world_mut().entity_mut(npc).add_child(child);

    // Spawn Monster at 50.0
    let _monster = app
        .world_mut()
        .spawn((
            Monster {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 50.0,
                max: 50.0,
            },
            SpawnIndex(0),
            Transform::from_xyz(50.0, 0.0, 0.0),
        ))
        .id();

    app.world_mut().resource_mut::<PortalSpawnTracker>().0 = 1;

    // 1. Decision
    app.update();

    // 2. Fire projectile (Advance 1.1s)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update(); // Should spawn projectile

    // Verify Projectile Exists
    let mut query = app.world_mut().query::<&Projectile>();
    let projectile_count = query.iter(app.world()).count();
    assert_eq!(projectile_count, 1, "Should spawn 1 projectile");
}
