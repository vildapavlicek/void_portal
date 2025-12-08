use {
    crate::{
        configs::SoldierConfig,
        portal::{Enemy, Health, PortalSpawnTracker, SpawnIndex},
    },
    bevy::{prelude::*, window::PrimaryWindow},
};

#[derive(Component)]
pub struct Soldier {
    pub attack_timer: Timer,
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct AttackRange(pub f32);

#[derive(Component)]
pub struct Moving(pub Entity);

#[derive(Component)]
pub struct Attacking(pub Entity);

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: Timer,
}

pub fn spawn_soldier(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    soldier_config: Res<SoldierConfig>,
    soldier_query: Query<Entity, With<Soldier>>,
) {
    if !soldier_query.is_empty() {
        return;
    }

    if let Some(window) = window_query.iter().next() {
        // Bottom 25% of Y, Middle of X
        // Window coords: Center (0,0). Top H/2, Bottom -H/2.
        // Bottom 25% range: [-H/2, -H/2 + H/4] = [-H/2, -H/4]
        // Middle of this range: -H/2 + H/8

        let half_height = window.height() / 2.0;
        let soldier_y = -half_height + (window.height() * 0.125);

        commands.spawn((
            Sprite {
                color: Color::srgb(0.0, 1.0, 0.0), // Terminal Green
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(0.0, soldier_y, 0.0),
            Soldier {
                attack_timer: Timer::from_seconds(
                    soldier_config.attack_timer,
                    TimerMode::Repeating,
                ),
                target: None,
            },
            AttackRange(soldier_config.attack_range),
        ));
        info!("Soldier spawned at y={}", soldier_y);
    }
}

pub fn soldier_decision_logic(
    mut commands: Commands,
    mut soldier_query: Query<
        (Entity, &Transform, &mut Soldier, &AttackRange),
        (Without<Moving>, Without<Attacking>),
    >,
    enemy_query: Query<(Entity, &Transform, &SpawnIndex), With<Enemy>>,
    portal_tracker: Res<PortalSpawnTracker>,
) {
    let current_spawn_count = portal_tracker.0;

    for (entity, transform, mut soldier, attack_range) in soldier_query.iter_mut() {
        // 1. Verify/Acquire Target
        let mut target_valid = false;
        if let Some(target) = soldier.target {
            if enemy_query.get(target).is_ok() {
                target_valid = true;
            }
        }

        if !target_valid {
            soldier.target = enemy_query
                .iter()
                .max_by_key(|(_, _, index)| current_spawn_count.wrapping_sub(index.0))
                .map(|(e, _, _)| e);
        }

        // 2. Decide Action
        if let Some(target) = soldier.target {
            if let Ok((_, target_transform, _)) = enemy_query.get(target) {
                let distance = transform.translation.distance(target_transform.translation);
                if distance > attack_range.0 {
                    commands.entity(entity).insert(Moving(target));
                } else {
                    commands.entity(entity).insert(Attacking(target));
                }
            }
        }
    }
}

pub fn soldier_movement_logic(
    mut commands: Commands,
    time: Res<Time>,
    soldier_config: Res<SoldierConfig>,
    mut soldier_query: Query<
        (Entity, &mut Transform, &Moving, &AttackRange),
        (Without<Attacking>, Without<Enemy>),
    >,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (entity, mut soldier_transform, moving, attack_range) in soldier_query.iter_mut() {
        let target = moving.0;

        if let Ok(target_transform) = enemy_query.get(target) {
            let distance = soldier_transform
                .translation
                .distance(target_transform.translation);

            if distance > attack_range.0 {
                let direction = (target_transform.translation - soldier_transform.translation)
                    .normalize_or_zero();
                soldier_transform.translation +=
                    direction * soldier_config.move_speed * time.delta_secs();
            } else {
                // Arrived
                commands.entity(entity).remove::<Moving>();
            }
        } else {
            // Target invalid/despawned
            commands.entity(entity).remove::<Moving>();
        }
    }
}

pub fn soldier_attack_logic(
    mut commands: Commands,
    time: Res<Time>,
    soldier_config: Res<SoldierConfig>,
    mut soldier_query: Query<
        (Entity, &Transform, &mut Soldier, &Attacking, &AttackRange),
        Without<Moving>,
    >,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (entity, soldier_transform, mut soldier, attacking, attack_range) in
        soldier_query.iter_mut()
    {
        let target = attacking.0;

        if let Ok(target_transform) = enemy_query.get(target) {
            let distance = soldier_transform
                .translation
                .distance(target_transform.translation);

            if distance > attack_range.0 {
                // Target moved out of range
                commands.entity(entity).remove::<Attacking>();
            } else {
                // Process Attack
                soldier.attack_timer.tick(time.delta());
                if soldier.attack_timer.just_finished() {
                    let direction = (target_transform.translation - soldier_transform.translation)
                        .normalize_or_zero();
                    let speed = soldier_config.projectile_speed;

                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 0.0), // Yellow
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        Transform::from_translation(soldier_transform.translation),
                        Projectile {
                            velocity: direction * speed,
                            damage: soldier_config.projectile_damage,
                            lifetime: Timer::from_seconds(
                                soldier_config.projectile_lifetime,
                                TimerMode::Once,
                            ),
                        },
                    ));
                }
            }
        } else {
            // Target invalid/despawned
            commands.entity(entity).remove::<Attacking>();
        }
    }
}

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
    for (entity, mut transform, mut projectile) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += projectile.velocity * time.delta_secs();
    }
}

pub fn projectile_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
) {
    for (proj_entity, proj_transform, projectile) in projectile_query.iter() {
        let mut hit = false;
        for (_, enemy_transform, mut health) in enemy_query.iter_mut() {
            let distance = proj_transform
                .translation
                .distance(enemy_transform.translation);
            // Enemy size is 24, Projectile 8. Radius approx 12 + 4 = 16. Use 20 for buffer.
            if distance < 20.0 {
                health.current -= projectile.damage;
                hit = true;
                break; // One projectile hits one enemy
            }
        }

        if hit {
            commands.entity(proj_entity).despawn();
        }
    }
}
