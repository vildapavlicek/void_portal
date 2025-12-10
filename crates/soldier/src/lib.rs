use {
    bevy::{prelude::*, window::PrimaryWindow},
    bevy_common_assets::ron::RonAssetPlugin,
    common::GameState,
    enemy::{Enemy, Health, SpawnIndex},
    portal::PortalSpawnTracker,
    serde::Deserialize,
};

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SoldierConfig>::new(&["soldier.ron"]));

        app.register_type::<Soldier>()
            .register_type::<AttackRange>()
            .register_type::<Moving>()
            .register_type::<Attacking>()
            .register_type::<Projectile>()
            .register_type::<SoldierConfig>();

        app.add_systems(OnEnter(GameState::Playing), spawn_soldier);

        app.add_systems(
            Update,
            (
                soldier_movement_logic,
                soldier_decision_logic,
                soldier_attack_logic,
                move_projectiles,
                projectile_collision,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Configs
#[derive(Deserialize, Asset, Clone, Debug, Resource, Reflect)]
pub struct SoldierConfig {
    pub attack_timer: f32,
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_lifetime: f32,
    pub attack_range: f32,
    pub move_speed: f32,
}

// Components
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Soldier {
    pub attack_timer: Timer,
    pub target: Option<Entity>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AttackRange(pub f32);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Moving(pub Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Attacking(pub Entity);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: Timer,
}

// Systems
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
        let mut target_valid = false;
        if let Some(target) = soldier.target {
            if enemy_query.get(target).is_ok() {
                target_valid = true;
            }
        }

        let old_target = soldier.target;

        if !target_valid {
            soldier.target = enemy_query
                .iter()
                .max_by_key(|(_, _, index)| current_spawn_count.wrapping_sub(index.0))
                .map(|(e, _, _)| e);
        }

        if soldier.target.is_some() && soldier.target != old_target {
            let duration = soldier.attack_timer.duration();
            soldier.attack_timer.set_elapsed(duration);
        }

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

                let new_distance = soldier_transform
                    .translation
                    .distance(target_transform.translation);
                if new_distance <= attack_range.0 {
                    commands.entity(entity).remove::<Moving>();
                }
            } else {
                commands.entity(entity).remove::<Moving>();
            }
        } else {
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
                commands.entity(entity).remove::<Attacking>();
            } else {
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
                break;
            }
        }

        if hit {
            commands.entity(proj_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests_logic;
#[cfg(test)]
mod tests_timing;
