#![allow(clippy::type_complexity)]

use {
    bevy::{prelude::*, scene::DynamicScene},
    common::{events::DamageMessage, GameState},
    enemy::{Enemy, SpawnIndex},
    items::{
        AttackRange as ItemAttackRange, BaseDamage, Melee, ProjectileStats as ItemProjectileStats,
        Ranged,
    },
    portal::PortalSpawnTracker,
};

pub struct PlayerNpcsPlugin;

impl Plugin for PlayerNpcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageMessage>();
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
                (
                    player_npc_movement_logic,
                    melee_attack_emit,
                    ranged_attack_logic,
                ),
                (move_projectiles, projectile_collision),
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
pub enum Intent {
    #[default]
    Idle,
    MoveTo(Vec3),
    Attack(Entity),
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Debug, Component, Reflect, Default)]
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: Timer,
    pub source: Entity,
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
    mut player_npc_query: Query<
        (
            Entity,
            &mut Intent,
            &mut Target,
            &Transform,
            Option<&Children>,
        ),
        With<PlayerNpc>,
    >,
    weapon_query: Query<&ItemAttackRange, With<Weapon>>,
    enemy_query: Query<(Entity, &SpawnIndex, &Transform), With<Enemy>>,
    portal_tracker: Res<PortalSpawnTracker>,
) {
    let current_spawn_count = portal_tracker.0;

    for (_npc_entity, mut intent, mut target_comp, npc_transform, children) in
        player_npc_query.iter_mut()
    {
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
                .max_by_key(|(_, index, _)| current_spawn_count.wrapping_sub(index.0))
                .map(|(e, _, _)| e);
            info!(?target_comp, "found valid target");
        }

        // Decision logic based on target
        let Some(target_entity) = target_comp.0 else {
            *intent = Intent::Idle;
            continue;
        };

        if let Ok((_, _, target_transform)) = enemy_query.get(target_entity) {
            let distance = npc_transform
                .translation
                .distance(target_transform.translation);

            if distance <= max_range {
                *intent = Intent::Attack(target_entity);
            } else {
                *intent = Intent::MoveTo(target_transform.translation);
            }
        } else {
            // Target dead or gone
            info!("setting intent to idle");
            *intent = Intent::Idle;
        }
    }
}

pub fn player_npc_movement_logic(
    time: Res<Time>,
    mut player_npc_query: Query<(&mut Transform, &Intent, &MovementSpeed), With<PlayerNpc>>,
) {
    for (mut transform, intent, speed) in player_npc_query.iter_mut() {
        if let Intent::MoveTo(target_pos) = intent {
            let dir = (*target_pos - transform.translation).normalize_or_zero();
            transform.translation += dir * speed.0 * time.delta_secs();
        }
    }
}

pub fn melee_attack_emit(
    time: Res<Time>,
    player_npc_query: Query<(Entity, &Intent, &Children), With<PlayerNpc>>,
    mut weapon_query: Query<
        (&mut WeaponCooldown, &ItemAttackRange, &BaseDamage),
        (With<Weapon>, With<Melee>),
    >,
    mut damage_events: MessageWriter<DamageMessage>,
) {
    for (npc_entity, intent, children) in player_npc_query.iter() {
        if let Intent::Attack(target_entity) = intent {
            for child in children.iter() {
                if let Ok((mut cooldown, _range, damage)) = weapon_query.get_mut(child) {
                    cooldown.timer.tick(time.delta());

                    if cooldown.timer.just_finished() {
                        // EMIT MESSAGE
                        damage_events.write(DamageMessage {
                            source: npc_entity,
                            target: *target_entity,
                            amount: damage.0,
                            damage_type: common::events::DamageType::Physical,
                        });
                    }
                }
            }
        }
    }
}

pub fn ranged_attack_logic(
    mut commands: Commands,
    time: Res<Time>,
    player_npc_query: Query<(Entity, &Transform, &Intent, &Children), With<PlayerNpc>>,
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
    for (npc_entity, npc_tf, intent, children) in player_npc_query.iter() {
        let Intent::Attack(target_entity) = intent else {
            continue;
        };

        let Ok(target_tf) = enemy_query.get(*target_entity) else {
            continue;
        };

        for child in children.iter() {
            if let Ok((mut cooldown, _range, damage, proj_stats)) = weapon_query.get_mut(child) {
                cooldown.timer.tick(time.delta());

                if cooldown.timer.just_finished() {
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
                            source: npc_entity,
                        },
                    ));
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
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: MessageWriter<DamageMessage>,
) {
    for (proj_entity, proj_transform, projectile) in projectile_query.iter() {
        let mut hit = false;
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let distance = proj_transform
                .translation
                .distance(enemy_transform.translation);
            // Enemy size is 24, Projectile 8. Radius approx 12 + 4 = 16. Use 20 for buffer.
            if distance < 20.0 {
                damage_events.write(DamageMessage {
                    source: projectile.source,
                    target: enemy_entity,
                    amount: projectile.damage,
                    damage_type: common::events::DamageType::Physical,
                });
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
