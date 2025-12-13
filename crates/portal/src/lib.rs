#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use {
    bevy::{prelude::*, window::PrimaryWindow},
    bevy_common_assets::ron::RonAssetPlugin,
    common::{
        ChangeActiveLevel, GameState, GrowthStrategy, RequestUpgrade, Reward, ScavengeModifier,
        UpgradePortal, UpgradeableStat,
    },
    enemy::{AvailableEnemies, Enemy, Health, Lifetime, SpawnIndex, Speed},
    rand::Rng,
    std::collections::HashMap,
    wallet::Wallet,
};

mod config;
pub use config::*;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<PortalConfig>::new(&["portal.ron"]));

        app.add_message::<ChangeActiveLevel>();

        app.register_type::<Portal>()
            .register_type::<UpgradeSlot>()
            .register_type::<UpgradeableStat>()
            .register_type::<PortalConfig>()
            .register_type::<PortalStats>()
            .register_type::<SpawnTimer>()
            .register_type::<PortalUpgrades>();

        app.init_resource::<PortalSpawnTracker>();

        app.add_systems(OnEnter(GameState::Playing), spawn_portal);

        app.add_systems(
            Update,
            (
                spawn_enemies,
                handle_portal_upgrade,
                handle_generic_upgrades,
                handle_active_level_change,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Components
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Portal {
    pub max_unlocked_level: u32,
    pub active_level: u32,
    pub upgrade_price: f32,
    pub price_strategy: GrowthStrategy,
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct UpgradeSlot {
    pub name: String,
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct PortalStats {
    pub stats: LevelScaledStats,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnTimer(pub Timer);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PortalUpgrades(pub HashMap<String, Entity>);

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

// Systems
pub fn spawn_portal(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    portal_query: Query<Entity, With<Portal>>,
    portal_config: Res<PortalConfig>,
) {
    if !portal_query.is_empty() {
        return;
    }

    if let Some(window) = window_query.iter().next() {
        let half_height = window.height() / 2.0;
        let portal_y = half_height - portal_config.portal_top_offset;

        // Calculate initial price (using level from config, likely 0)
        let initial_price = portal_config
            .level_up_price
            .calculate(portal_config.level as f32);

        // Calculate initial spawn time (based on active level, which is config.level initially)
        let initial_spawn_time = portal_config
            .level_scaled_stats
            .spawn_timer
            .calculate(portal_config.level as f32);

        let portal_entity = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.5, 0.0, 0.5), // Purple
                    custom_size: Some(Vec2::new(16.0, 32.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, portal_y, 0.0),
                Portal {
                    max_unlocked_level: portal_config.level,
                    active_level: portal_config.level,
                    upgrade_price: initial_price,
                    price_strategy: portal_config.level_up_price.clone(),
                },
                PortalStats {
                    stats: portal_config.level_scaled_stats.clone(),
                },
                SpawnTimer(Timer::from_seconds(
                    initial_spawn_time,
                    TimerMode::Repeating,
                )),
                Pickable::default(),
            ))
            .id();

        // Spawn upgrades as children
        let mut upgrades = HashMap::new();
        commands.entity(portal_entity).with_children(|parent| {
            for (name, config) in &portal_config.upgrades {
                // New logic: UpgradeableStat handles calculation
                let stat = UpgradeableStat::new(config.value.clone(), config.price.clone());

                let id = parent
                    .spawn((UpgradeSlot { name: name.clone() }, stat))
                    .id();
                upgrades.insert(name.clone(), id);
            }
        });

        commands
            .entity(portal_entity)
            .insert(PortalUpgrades(upgrades));

        info!(
            "Portal spawned at y={} | entity={portal_entity:?}",
            portal_y
        );
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    available_enemies: Res<AvailableEnemies>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut portal_query: Query<(
        &Transform,
        &Portal,
        &PortalUpgrades,
        &PortalStats,
        &mut SpawnTimer,
    )>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
    mut spawn_tracker: ResMut<PortalSpawnTracker>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    portal_config: Res<PortalConfig>,
) {
    for (portal_transform, portal, upgrades, portal_stats, mut spawn_timer) in
        portal_query.iter_mut()
    {
        spawn_timer.0.tick(time.delta());

        if spawn_timer.0.just_finished() {
            if available_enemies.0.is_empty() {
                warn!("No enemies available to spawn!");
                continue;
            }

            // Find Capacity and Lifetime upgrades
            let capacity_entity = upgrades
                .0
                .get("Capacity")
                .expect("Capacity upgrade not found in PortalUpgrades");
            let lifetime_entity = upgrades
                .0
                .get("Lifetime")
                .expect("Lifetime upgrade not found in PortalUpgrades");

            let (_, capacity_stat) = upgrade_query
                .get(*capacity_entity)
                .expect("Capacity upgrade entity not found in world");
            let (_, lifetime_stat) = upgrade_query
                .get(*lifetime_entity)
                .expect("Lifetime upgrade entity not found in world");

            let capacity_val = capacity_stat.value;
            let lifetime_bonus = lifetime_stat.value;

            // Check Global Capacity (Current Logic: Global Count vs Local Capacity)
            if enemy_query.iter().count() >= capacity_val as usize {
                // Max enemies reached for this portal's capacity check
                continue;
            }

            let Some(window) = window_query.iter().next() else {
                continue;
            };

            let enemy_config = &available_enemies.0[0];

            let half_width = window.width() / 2.0;
            let half_height = window.height() / 2.0;

            // Dynamic stats calculation using PortalStats component and GrowthStrategy
            // USES ACTIVE LEVEL
            let health_multiplier = portal_stats
                .stats
                .enemy_health
                .calculate(portal.active_level as f32);
            let reward_multiplier = portal_stats
                .stats
                .void_shards_reward
                .calculate(portal.active_level as f32);
            let lifetime_multiplier = portal_stats
                .stats
                .base_enemy_lifetime
                .calculate(portal.active_level as f32);
            let base_speed = portal_stats
                .stats
                .base_enemy_speed
                .calculate(portal.active_level as f32);

            let max_health = health_multiplier * enemy_config.health_coef;
            let speed = base_speed * enemy_config.speed_coef;
            let reward = reward_multiplier * enemy_config.reward_coef;

            // Lifetime = (Base Scaled * Coef) + Bonus Independent
            let lifetime_val = (lifetime_multiplier * enemy_config.lifetime_coef) + lifetime_bonus;

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
                    Transform::from_translation(portal_transform.translation),
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
                    ScavengeModifier(portal_config.scavenger_penalty_coef),
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
            info!("Enemy spawned! Target: {:?}", target_position);
        }
    }
}

pub fn handle_portal_upgrade(
    mut events: MessageReader<UpgradePortal>,
    mut portal_query: Query<(&mut Portal, &PortalStats, &mut SpawnTimer)>,
    mut wallet: ResMut<Wallet>,
) {
    for _event in events.read() {
        if let Some((mut portal, stats, mut spawn_timer)) = portal_query.iter_mut().next() {
            if wallet.void_shards >= portal.upgrade_price {
                wallet.void_shards -= portal.upgrade_price;

                let old_max_level = portal.max_unlocked_level;
                portal.max_unlocked_level += 1;

                // QoL: If active level was at max, snap to new max
                if portal.active_level == old_max_level {
                    portal.active_level = portal.max_unlocked_level;
                }

                // Update Price using Strategy (based on MAX level)
                portal.upgrade_price = portal
                    .price_strategy
                    .calculate(portal.max_unlocked_level as f32);

                // Update Spawn Timer (based on ACTIVE level)
                let new_spawn_time = stats
                    .stats
                    .spawn_timer
                    .calculate(portal.active_level as f32);
                spawn_timer
                    .0
                    .set_duration(std::time::Duration::from_secs_f32(new_spawn_time));

                info!(
                    "Portal upgraded to Max Level {}. New Price: {}",
                    portal.max_unlocked_level, portal.upgrade_price
                );
            } else {
                warn!("Not enough shards to upgrade portal!");
            }
        }
    }
}

pub fn handle_active_level_change(
    mut events: MessageReader<ChangeActiveLevel>,
    mut portal_query: Query<(&mut Portal, &PortalStats, &mut SpawnTimer)>,
) {
    for event in events.read() {
        if let Ok((mut portal, stats, mut spawn_timer)) = portal_query.get_mut(event.portal_entity)
        {
            let new_level = (portal.active_level as i32 + event.change)
                .clamp(1, portal.max_unlocked_level as i32) as u32;

            if new_level != portal.active_level {
                portal.active_level = new_level;

                // Update Spawn Timer for new active level
                let new_spawn_time = stats
                    .stats
                    .spawn_timer
                    .calculate(portal.active_level as f32);
                spawn_timer
                    .0
                    .set_duration(std::time::Duration::from_secs_f32(new_spawn_time));

                info!("Portal active level changed to {}", portal.active_level);
            }
        }
    }
}

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
