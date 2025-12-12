#![allow(clippy::type_complexity)]

use {
    bevy::{prelude::*, scene::DynamicScene, window::PrimaryWindow},
    bevy_common_assets::ron::RonAssetPlugin,
    common::GameState,
    enemy::{Enemy, Health, SpawnIndex},
    items::{
        Armor, AttackRange as ItemAttackRange, AttackSpeed, BaseDamage, Item, Melee,
        ProjectileStats as ItemProjectileStats, Ranged,
    },
    portal::PortalSpawnTracker,
    serde::Deserialize,
    std::time::Duration,
};

pub struct PlayerNpcsPlugin;

impl Plugin for PlayerNpcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SoldierConfig>::new(&["soldier.ron"]));

        app.register_type::<Soldier>()
            .register_type::<AttackRange>()
            .register_type::<Moving>()
            .register_type::<Attacking>()
            .register_type::<Projectile>()
            .register_type::<SoldierConfig>()
            .register_type::<Equipment>()
            .register_type::<CombatStats>()
            .register_type::<BaseCombatStats>();

        app.add_systems(OnEnter(GameState::Playing), spawn_player_npc);

        app.add_systems(
            Update,
            (
                // link_starting_equipment,
                recalculate_stats,
                player_npc_movement_logic,
                player_npc_decision_logic,
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

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Equipment {
    pub main_hand: Option<Entity>,
    pub armor: Option<Entity>,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CombatStats {
    pub damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub move_speed: f32,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct BaseCombatStats {
    pub damage: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub move_speed: f32,
}

// Systems
pub fn link_starting_equipment(
    mut query: Query<(Entity, &Children, &mut Equipment), (With<Soldier>, Added<Soldier>)>,
    item_query: Query<Entity, With<Item>>,
) {
    for (soldier, children, mut equip) in query.iter_mut() {
        for child in children.iter() {
            if item_query.contains(child) {
                equip.main_hand = Some(child);
                info!(
                    "Auto-linked starting weapon {:?} to soldier {:?}",
                    child, soldier
                );
                break;
            }
        }
    }
}

pub fn recalculate_stats(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut CombatStats,
            &Equipment,
            &mut Soldier,
            &mut AttackRange,
            Option<&BaseCombatStats>,
        ),
        Changed<Equipment>,
    >,
    item_query: Query<
        (
            Option<&BaseDamage>,
            Option<&ItemAttackRange>,
            Option<&AttackSpeed>,
            Option<&ItemProjectileStats>,
            Option<&Melee>,
            Option<&Ranged>,
        ),
        With<Item>,
    >,
    armor_query: Query<&Armor>,
    config: Res<SoldierConfig>,
) {
    for (entity, mut stats, equip, mut soldier, mut attack_range, base_stats) in query.iter_mut() {
        // 1. Reset to Base
        if let Some(base) = base_stats {
            stats.damage = base.damage;
            stats.attack_range = base.attack_range;
            stats.attack_cooldown = base.attack_cooldown;
            stats.projectile_speed = base.projectile_speed;
            stats.projectile_lifetime = base.projectile_lifetime;
            stats.move_speed = base.move_speed;
        } else {
            stats.damage = config.projectile_damage;
            stats.attack_range = config.attack_range;
            stats.attack_cooldown = config.attack_timer;
            stats.projectile_speed = config.projectile_speed;
            stats.projectile_lifetime = config.projectile_lifetime;
            stats.move_speed = config.move_speed;
        }

        // Cleanup markers
        commands.entity(entity).remove::<Melee>();
        commands.entity(entity).remove::<Ranged>();

        let mut is_melee = false;
        let mut is_ranged = false;

        // 2. Apply Weapon
        if let Some(e) = equip.main_hand {
            if let Ok((damage, range, speed, proj_stats, melee, ranged)) = item_query.get(e) {
                if let Some(d) = damage {
                    stats.damage = d.0; // Override or Add? Assuming Override/Set Base for now as per clean RPG stats usually
                }
                if let Some(r) = range {
                    stats.attack_range = r.0;
                }
                if let Some(s) = speed {
                    stats.attack_cooldown = s.0;
                }
                if let Some(p) = proj_stats {
                    stats.projectile_speed = p.speed;
                    stats.projectile_lifetime = p.lifetime;
                }

                if melee.is_some() {
                    commands.entity(entity).insert(Melee);
                    is_melee = true;
                }
                if ranged.is_some() {
                    commands.entity(entity).insert(Ranged);
                    is_ranged = true;
                }
            }
        }

        // 3. Apply Armor
        if let Some(e) = equip.armor {
            if let Ok(a) = armor_query.get(e) {
                stats.move_speed += a.movement_speed_modifier;
            }
        }

        // Default behavior if no specific marker set (e.g. no weapon or unassigned)
        if !is_melee && !is_ranged {
            // Default to Ranged as per original Soldier design
            commands.entity(entity).insert(Ranged);
        }

        // 4. Update Dependent Components
        soldier
            .attack_timer
            .set_duration(Duration::from_secs_f32(stats.attack_cooldown));
        attack_range.0 = stats.attack_range;
    }
}

pub fn spawn_player_npc(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    player_npc_query: Query<Entity, With<Soldier>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if !player_npc_query.is_empty() {
        return;
    }

    // if let Some(window) = window_query.iter().next() {
    //     let half_height = window.height() / 2.0;
    //     let player_npc_y = -half_height + (window.height() * 0.125);

    //     let soldier_handle =
    //         asset_server.load::<DynamicScene>("prefabs/player_npcs/soldier.scn.ron");
    // }

    let soldier_handle = asset_server.load::<DynamicScene>("prefabs/player_npcs/soldier.scn.ron");
    scene_spawner.spawn_dynamic(soldier_handle);
}

pub fn player_npc_decision_logic(
    mut commands: Commands,
    mut player_npc_query: Query<
        (Entity, &Transform, &mut Soldier, &AttackRange),
        (Without<Moving>, Without<Attacking>),
    >,
    enemy_query: Query<(Entity, &Transform, &SpawnIndex), With<Enemy>>,
    portal_tracker: Res<PortalSpawnTracker>,
) {
    let current_spawn_count = portal_tracker.0;

    for (entity, transform, mut soldier, attack_range) in player_npc_query.iter_mut() {
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

pub fn player_npc_movement_logic(
    mut commands: Commands,
    time: Res<Time>,
    mut player_npc_query: Query<
        (Entity, &mut Transform, &Moving, &AttackRange, &CombatStats),
        (Without<Attacking>, Without<Enemy>),
    >,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (entity, mut player_npc_transform, moving, attack_range, combat_stats) in
        player_npc_query.iter_mut()
    {
        let target = moving.0;

        if let Ok(target_transform) = enemy_query.get(target) {
            let distance = player_npc_transform
                .translation
                .distance(target_transform.translation);

            if distance > attack_range.0 {
                let direction = (target_transform.translation - player_npc_transform.translation)
                    .normalize_or_zero();
                player_npc_transform.translation +=
                    direction * combat_stats.move_speed * time.delta_secs();

                let new_distance = player_npc_transform
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

pub fn melee_attack_logic(
    mut commands: Commands,
    time: Res<Time>,
    mut player_npc_query: Query<
        (Entity, &Transform, &mut Soldier, &Attacking, &CombatStats),
        (With<Melee>, Without<Moving>),
    >,
    mut enemy_query: Query<(&Transform, &mut Health), With<Enemy>>,
) {
    for (entity, player_transform, mut soldier, attacking, stats) in player_npc_query.iter_mut() {
        let target = attacking.0;

        if let Ok((target_transform, mut health)) = enemy_query.get_mut(target) {
            // Check range again just in case (optional but good for strictness)
            let distance = player_transform
                .translation
                .distance(target_transform.translation);
            if distance > stats.attack_range {
                commands.entity(entity).remove::<Attacking>();
                continue;
            }

            soldier.attack_timer.tick(time.delta());
            if soldier.attack_timer.just_finished() {
                // Instant Hit
                health.current -= stats.damage;
                // Visual effect? (Optional)
                info!("Melee hit for {} damage!", stats.damage);
            }
        } else {
            commands.entity(entity).remove::<Attacking>();
        }
    }
}

pub fn ranged_attack_logic(
    mut commands: Commands,
    time: Res<Time>,
    mut player_npc_query: Query<
        (Entity, &Transform, &mut Soldier, &Attacking, &CombatStats),
        (With<Ranged>, Without<Moving>),
    >,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (entity, player_npc_transform, mut soldier, attacking, combat_stats) in
        player_npc_query.iter_mut()
    {
        let target = attacking.0;

        if let Ok(target_transform) = enemy_query.get(target) {
            let distance = player_npc_transform
                .translation
                .distance(target_transform.translation);

            if distance > combat_stats.attack_range {
                commands.entity(entity).remove::<Attacking>();
            } else {
                soldier.attack_timer.tick(time.delta());
                if soldier.attack_timer.just_finished() {
                    let direction = (target_transform.translation
                        - player_npc_transform.translation)
                        .normalize_or_zero();

                    let speed = combat_stats.projectile_speed;
                    // Use configured lifetime or fallback
                    let lifetime_secs = if combat_stats.projectile_lifetime > 0.0 {
                        combat_stats.projectile_lifetime
                    } else if speed > 0.0 {
                        combat_stats.attack_range / speed
                    } else {
                        0.0
                    };

                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 0.0), // Yellow
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        Transform::from_translation(player_npc_transform.translation),
                        Projectile {
                            velocity: direction * speed,
                            damage: combat_stats.damage,
                            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                        },
                    ));
                }
            }
        } else {
            commands.entity(entity).remove::<Attacking>();
        }
    }
}

// Deprecated generic attack logic removed/replaced by specific ones.

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
mod tests_items;
