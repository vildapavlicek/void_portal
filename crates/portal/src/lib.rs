#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use {
    bevy::prelude::*,
    common::{
        ChangeActiveLevel, GameState, RequestUpgrade, SpawnEnemyRequest, UpgradePortal,
        UpgradeableStat, Reward, ScavengeModifier,
    },
    enemy::{AvailableEnemies, Enemy, Health, Lifetime, SpawnIndex, Speed},
    std::collections::HashMap,
    wallet::Wallet,
    rand::Rng,
};

mod components;
pub use components::*;

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct UpgradeSlot {
    pub name: String,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PortalUpgrades(pub HashMap<String, Entity>);

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChangeActiveLevel>();

        app.register_type::<PortalRoot>()
            .register_type::<PortalLevel>()
            .register_type::<UpgradeCost>()
            .register_type::<PortalSpawner>()
            .register_type::<EnemyScaling>()
            .register_type::<ScavengerPenalty>()
            .register_type::<UpgradeSlot>()
            .register_type::<UpgradeableStat>()
            .register_type::<PortalUpgrades>();

        app.init_resource::<PortalSpawnTracker>();

        app.add_message::<SpawnEnemyRequest>();

        app.add_systems(
            Update,
            (
                layout_portal,
                (portal_tick_logic, portal_spawn_logic).chain(),
                handle_portal_upgrade,
                handle_generic_upgrades,
                handle_active_level_change,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

// Systems

// A. Layout
pub fn layout_portal(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut portal_query: Query<&mut Transform, (With<PortalRoot>, Changed<Transform>)>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    // Hardcoded offset as per plan/requirements
    let top_offset = 100.0;
    let portal_y = (window.height() / 2.0) - top_offset;

    for mut transform in portal_query.iter_mut() {
        if (transform.translation.y - portal_y).abs() > 0.01 {
            transform.translation.y = portal_y;
        }
    }
}

// B. Tick Logic
pub fn portal_tick_logic(
    time: Res<Time>,
    mut portal_query: Query<(Entity, &mut PortalSpawner, &Children)>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
    enemy_query: Query<(), With<Enemy>>,
    available_enemies: Res<AvailableEnemies>,
    mut spawn_events: MessageWriter<SpawnEnemyRequest>,
) {
    let current_enemy_count = enemy_query.iter().count();

    for (entity, mut spawner, children) in portal_query.iter_mut() {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() {
            if available_enemies.0.is_empty() {
                warn!("No enemies available to spawn!");
                continue;
            }

            // Iterate children to find "Capacity"
            let capacity = children.iter()
                .filter_map(|child| upgrade_query.get(child).ok())
                .find(|(slot, _)| slot.name == "Capacity");

            if let Some((_, cap_stat)) = capacity {
                if current_enemy_count < cap_stat.value as usize {
                     spawn_events.write(SpawnEnemyRequest {
                        portal_entity: entity,
                    });
                }
            }
        }
    }
}

// C. Spawn Logic
pub fn portal_spawn_logic(
    mut commands: Commands,
    mut events: MessageReader<SpawnEnemyRequest>,
    portal_query: Query<(&Transform, &PortalLevel, &EnemyScaling, &Children, Option<&ScavengerPenalty>)>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
    available_enemies: Res<AvailableEnemies>,
    mut spawn_tracker: ResMut<PortalSpawnTracker>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if events.is_empty() { return; }

    let Some(window) = window_query.iter().next() else { return; };
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    let enemy_config = &available_enemies.0[0]; // Simplified selection

    for request in events.read() {
        let Ok((portal_tf, level, scaling, children, scav_penalty_opt)) = portal_query.get(request.portal_entity) else {
            continue;
        };

        let scavenger_penalty = scav_penalty_opt.map(|p| p.0).unwrap_or(1.0);

        // Find Lifetime upgrade
        let lifetime_upgrade = children.iter()
            .filter_map(|child| upgrade_query.get(child).ok())
            .find(|(slot, _)| slot.name == "Lifetime");

        let bonus_lifetime = if let Some((_, stat)) = lifetime_upgrade {
            stat.value
        } else {
            0.0
        };

        // Calculate Stats
        // 1. Base from Portal (Active Level)
        let base_health = scaling.health_strategy.calculate(level.active as f32);
        let base_reward = scaling.reward_strategy.calculate(level.active as f32);
        let base_speed = scaling.speed_strategy.calculate(level.active as f32);
        let base_lifetime = scaling.lifetime_strategy.calculate(level.active as f32);

        // 2. Apply Enemy Config Coefficients
        let max_health = base_health * enemy_config.health_coef;
        let speed = base_speed * enemy_config.speed_coef;
        let reward = base_reward * enemy_config.reward_coef;

        // Lifetime = (Base * Coef) + Bonus
        let lifetime_val = (base_lifetime * enemy_config.lifetime_coef) + bonus_lifetime;

        let mut rng = rand::rng();
        let target_x = rng.random_range(-half_width..half_width);
        let target_y = rng.random_range(-half_height..half_height);
        let target_position = Vec2::new(target_x, target_y);

        commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.0, 1.0), // Blue
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
                Transform::from_translation(portal_tf.translation),
                Enemy { target_position },
                SpawnIndex(spawn_tracker.0),
                Health {
                    current: max_health,
                    max: max_health,
                },
                Lifetime {
                    timer: Timer::from_seconds(lifetime_val, TimerMode::Once),
                },
                Reward(reward),
                Speed(speed),
                ScavengeModifier(scavenger_penalty),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(format!("{:.0}", max_health)),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(0.0, 20.0, 1.0)),
                ));
            });

        spawn_tracker.0 = spawn_tracker.0.wrapping_add(1);
        info!(
            "Enemy spawned via Event Loop from Portal {:?}",
            request.portal_entity
        );
    }
}

// D. Upgrade Portal
pub fn handle_portal_upgrade(
    mut events: MessageReader<UpgradePortal>,
    mut portal_query: Query<(&mut PortalLevel, &mut UpgradeCost, &mut PortalSpawner)>,
    mut wallet: ResMut<Wallet>,
) {
    for _ in events.read() {
        if let Some((mut level, mut cost, mut spawner)) = portal_query.iter_mut().next() {
            if wallet.void_shards >= cost.current_price {
                wallet.void_shards -= cost.current_price;

                level.max_unlocked += 1;
                level.active = level.max_unlocked; // Auto-snap

                // Recalculate Price
                cost.current_price = cost.strategy.calculate(level.max_unlocked as f32);

                // Recalculate Spawn Timer
                let new_time = spawner.interval_strategy.calculate(level.active as f32);
                spawner.timer.set_duration(std::time::Duration::from_secs_f32(new_time));

                info!("Portal upgraded to Max Level {}. New Price: {}", level.max_unlocked, cost.current_price);
            } else {
                warn!("Not enough shards to upgrade portal!");
            }
        }
    }
}

// E. Active Level Change
pub fn handle_active_level_change(
    mut events: MessageReader<ChangeActiveLevel>,
    mut portal_query: Query<(&mut PortalLevel, &mut PortalSpawner)>,
) {
    for event in events.read() {
        if let Ok((mut level, mut spawner)) = portal_query.get_mut(event.portal_entity) {
             let new_level = (level.active as i32 + event.change)
                .clamp(1, level.max_unlocked as i32) as u32;

            if new_level != level.active {
                level.active = new_level;

                // Recalculate Spawn Timer
                let new_time = spawner.interval_strategy.calculate(level.active as f32);
                spawner.timer.set_duration(std::time::Duration::from_secs_f32(new_time));

                 info!("Portal active level changed to {}", level.active);
            }
        }
    }
}

// F. Generic Upgrades
pub fn handle_generic_upgrades(
    mut events: MessageReader<RequestUpgrade>,
    mut upgrade_query: Query<(&mut UpgradeableStat, &UpgradeSlot)>,
    mut wallet: ResMut<Wallet>,
) {
    for event in events.read() {
        if let Ok((mut stat, slot)) = upgrade_query.get_mut(event.upgrade_entity) {
            if wallet.void_shards >= stat.price {
                wallet.void_shards -= stat.price;
                stat.upgrade();

                info!(
                    "Upgrade '{}' upgraded to {}. New Price: {}",
                    slot.name, stat.value, stat.price
                );
            } else {
                 warn!(
                    "Not enough shards to upgrade '{}' (Cost: {})",
                    slot.name, stat.price
                );
            }
        } else {
            warn!("Upgrade entity {:?} not found!", event.upgrade_entity);
        }
    }
}

#[cfg(test)]
mod tests_mechanics;
