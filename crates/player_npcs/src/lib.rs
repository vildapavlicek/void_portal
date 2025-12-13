#![allow(clippy::type_complexity)]

use {
    bevy::{prelude::*, scene::DynamicScene},
    common::GameState,
    enemy::{Enemy, Health, SpawnIndex},
    items::{
        AttackRange as ItemAttackRange, BaseDamage, Melee, ProjectileStats as ItemProjectileStats,
        Ranged,
    },
    portal::PortalSpawnTracker,
};

pub struct PlayerNpcsPlugin;

impl Plugin for PlayerNpcsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerNpc>()
            .register_type::<MovementSpeed>()
            .register_type::<Target>()
            .register_type::<Weapon>()
            .register_type::<WeaponCooldown>()
            .register_type::<Projectile>();

        app.add_systems(OnEnter(GameState::Playing), spawn_player_npc);

        app.add_systems(
            Update,
            (
                player_npc_decision_logic,
                player_npc_movement_logic,
                melee_attack_logic,
                ranged_attack_logic,
                move_projectiles,
                projectile_collision,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Components

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerNpc;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Target(pub Option<Entity>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Weapon;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WeaponCooldown {
    pub timer: Timer,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: Timer,
}

// Systems

pub fn spawn_player_npc(
    player_npc_query: Query<Entity, With<PlayerNpc>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    // Only spawn if not already present (simple logic for now, or maybe we want multiple?)
    // The previous logic checked `player_npc_query.is_empty()`. I'll keep that behavior.
    if !player_npc_query.is_empty() {
        return;
    }

    let soldier_handle = asset_server.load::<DynamicScene>("prefabs/player_npcs/soldier.scn.ron");
    scene_spawner.spawn_dynamic(soldier_handle);
}

pub fn player_npc_decision_logic(
    mut player_npc_query: Query<(Entity, &mut Target, Option<&Children>), With<PlayerNpc>>,
    weapon_query: Query<&ItemAttackRange, With<Weapon>>,
    enemy_query: Query<(Entity, &SpawnIndex), With<Enemy>>,
    portal_tracker: Res<PortalSpawnTracker>,
) {
    let current_spawn_count = portal_tracker.0;

    for (_npc_entity, mut target_comp, children) in player_npc_query.iter_mut() {
        // 1. Calculate Effective Range
        let mut max_range = 0.0;
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(range) = weapon_query.get(child) {
                    if range.0 > max_range {
                        max_range = range.0;
                    }
                }
            }
        }

        let mut target_valid = false;
        if let Some(target) = target_comp.0 {
            if enemy_query.get(target).is_ok() {
                target_valid = true;
            }
        }

        if !target_valid {
            target_comp.0 = enemy_query
                .iter()
                .max_by_key(|(_, index)| current_spawn_count.wrapping_sub(index.0))
                .map(|(e, _)| e);
        }
    }
}

pub fn player_npc_movement_logic(
    time: Res<Time>,
    mut player_npc_query: Query<
        (&mut Transform, &Target, &MovementSpeed, Option<&Children>),
        (With<PlayerNpc>, Without<Enemy>),
    >,
    weapon_query: Query<&ItemAttackRange, With<Weapon>>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (mut player_transform, target_comp, speed, children) in player_npc_query.iter_mut() {
        let Some(target) = target_comp.0 else {
            continue;
        };
        let Ok(target_transform) = enemy_query.get(target) else {
            continue;
        };

        // Calculate Effective Range
        let mut effective_range = 0.0;
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(range) = weapon_query.get(child) {
                    if range.0 > effective_range {
                        effective_range = range.0;
                    }
                }
            }
        }

        let distance = player_transform
            .translation
            .distance(target_transform.translation);

        if distance > effective_range {
            let direction =
                (target_transform.translation - player_transform.translation).normalize_or_zero();
            player_transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}

pub fn melee_attack_logic(
    time: Res<Time>,
    player_npc_query: Query<(Entity, &Transform, &Target, &Children), With<PlayerNpc>>,
    mut weapon_query: Query<
        (&mut WeaponCooldown, &ItemAttackRange, &BaseDamage),
        (With<Weapon>, With<Melee>),
    >,
    mut enemy_query: Query<(&Transform, &mut Health), With<Enemy>>,
) {
    for (npc_entity, npc_tf, target_comp, children) in player_npc_query.iter() {
        let Some(target_entity) = target_comp.0 else {
            continue;
        };
        let Ok((target_tf, mut target_health)) = enemy_query.get_mut(target_entity) else {
            warn!("target entity doesn't exist, or is missing required components");
            continue;
        };

        let distance = npc_tf.translation.distance(target_tf.translation);

        for child in children.iter() {
            match weapon_query.get_mut(child) {
                Ok((mut cooldown, range, damage)) => {
                    cooldown.timer.tick(time.delta());

                    if cooldown.timer.just_finished() {
                        // Check individual weapon range
                        if distance <= range.0 {
                            // Instant Hit
                            target_health.current -= damage.0;
                            info!(
                                "Melee hit from {:?} (Weapon {:?}) for {}",
                                npc_entity, child, damage.0
                            );
                        }
                    }
                }
                Err(err) => {
                    error!(%err, "no meelee weapon found");
                    return;
                }
            }
        }
    }
}

pub fn ranged_attack_logic(
    mut commands: Commands,
    time: Res<Time>,
    player_npc_query: Query<(Entity, &Transform, &Target, &Children), With<PlayerNpc>>,
    mut weapon_query: Query<
        (
            &mut WeaponCooldown,
            &ItemAttackRange,
            &BaseDamage,
            &ItemProjectileStats,
        ),
        (With<Weapon>, With<Ranged>),
    >,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (_npc_entity, npc_tf, target_comp, children) in player_npc_query.iter() {
        let Some(target_entity) = target_comp.0 else {
            continue;
        };
        let Ok(target_tf) = enemy_query.get(target_entity) else {
            continue;
        };

        let distance = npc_tf.translation.distance(target_tf.translation);

        for child in children.iter() {
            if let Ok((mut cooldown, range, damage, proj_stats)) = weapon_query.get_mut(child) {
                cooldown.timer.tick(time.delta());

                if cooldown.timer.just_finished() {
                    if distance <= range.0 {
                        let direction =
                            (target_tf.translation - npc_tf.translation).normalize_or_zero();

                        // Spawn Projectile
                        commands.spawn((
                            Sprite {
                                color: Color::srgb(1.0, 1.0, 0.0), // Yellow
                                custom_size: Some(Vec2::new(8.0, 8.0)),
                                ..default()
                            },
                            Transform::from_translation(npc_tf.translation),
                            Projectile {
                                velocity: direction * proj_stats.speed,
                                damage: damage.0,
                                lifetime: Timer::from_seconds(proj_stats.lifetime, TimerMode::Once),
                            },
                        ));
                    }
                }
            }
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
