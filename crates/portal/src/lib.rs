#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use {
    bevy::{prelude::*, window::PrimaryWindow},
    bevy_common_assets::ron::RonAssetPlugin,
    common::{GameState, GrowthStrategy, Reward, UpgradePortal, UpgradePortalCapacity, UpgradeableStat},
    enemy::{AvailableEnemies, Enemy, Health, Lifetime, SpawnIndex, Speed},
    rand::Rng,
    wallet::Wallet,
};

mod config;
pub use config::*;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<PortalConfig>::new(&["portal.ron"]));

        app.register_type::<Portal>()
            .register_type::<PortalCapacity>()
            .register_type::<PortalBonusLifetime>()
            .register_type::<PortalConfig>();

        app.init_resource::<PortalSpawnTracker>()
            .init_resource::<EnemySpawnTimer>();

        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_portal, init_enemy_spawn_timer),
        );

        app.add_systems(
            Update,
            (
                spawn_enemies,
                handle_portal_upgrade,
                handle_portal_capacity_upgrade,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Components
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Portal {
    pub level: u32,
    pub upgrade_price: f32,
    pub price_growth_factor: f32,
    pub price_growth_strategy: GrowthStrategy,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PortalCapacity(pub UpgradeableStat);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PortalBonusLifetime(pub UpgradeableStat);

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

#[derive(Resource, Default)]
pub struct EnemySpawnTimer(pub Timer);

// Systems
fn init_enemy_spawn_timer(mut commands: Commands, portal_config: Res<PortalConfig>) {
    commands.insert_resource(EnemySpawnTimer(Timer::from_seconds(
        portal_config.level_scaled_stats.spawn_timer.value,
        TimerMode::Repeating,
    )));
}

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

        let capacity = UpgradeableStat::new(
            portal_config.independently_leveled_stats.capacity.value,
            portal_config.independently_leveled_stats.capacity.price,
            portal_config.independently_leveled_stats.capacity.growth_factor,
            portal_config.independently_leveled_stats.capacity.growth_strategy,
            portal_config.independently_leveled_stats.capacity.price_growth_factor,
            portal_config.independently_leveled_stats.capacity.price_growth_strategy,
        );

        let lifetime = UpgradeableStat::new(
            portal_config.independently_leveled_stats.lifetime.value,
            portal_config.independently_leveled_stats.lifetime.price,
            portal_config.independently_leveled_stats.lifetime.growth_factor,
            portal_config.independently_leveled_stats.lifetime.growth_strategy,
            portal_config.independently_leveled_stats.lifetime.price_growth_factor,
            portal_config.independently_leveled_stats.lifetime.price_growth_strategy,
        );

        let entity = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.5, 0.0, 0.5), // Purple
                    custom_size: Some(Vec2::new(16.0, 32.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, portal_y, 0.0),
                Portal {
                    level: portal_config.level,
                    upgrade_price: portal_config.level_up_price.value,
                    price_growth_factor: portal_config.level_up_price.growth_factor,
                    price_growth_strategy: portal_config.level_up_price.growth_strategy,
                },
                PortalCapacity(capacity),
                PortalBonusLifetime(lifetime),
                Pickable::default(),
            ))
            .id();
        info!("Portal spawned at y={} | entity={entity:?}", portal_y);
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    portal_config: Res<PortalConfig>,
    available_enemies: Res<AvailableEnemies>,
    enemy_query: Query<Entity, With<Enemy>>,
    portal_query: Query<(&Transform, &Portal, &PortalCapacity, &PortalBonusLifetime)>,
    mut spawn_tracker: ResMut<PortalSpawnTracker>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        if available_enemies.0.is_empty() {
            warn!("No enemies available to spawn!");
            return;
        }

        let Some((portal_transform, portal, portal_capacity, portal_lifetime)) = portal_query.iter().next()
        else {
            warn!("No portal found to spawn enemies from");
            return;
        };

        let enemy_config = &available_enemies.0[0];

        if enemy_query.iter().count() >= portal_capacity.0.value as usize {
            info!("Max enemies reached, skipping spawn");
            return;
        }

        let Some(window) = window_query.iter().next() else {
            return;
        };

        let half_width = window.width() / 2.0;
        let half_height = window.height() / 2.0;

        // Dynamic stats calculation based on Portal Level
        let health_multiplier = portal_config.level_scaled_stats.enemy_health.calculate(portal.level);
        let reward_multiplier = portal_config.level_scaled_stats.void_shards_reward.calculate(portal.level);
        let lifetime_multiplier = portal_config.level_scaled_stats.base_enemy_lifetime.calculate(portal.level);

        let base_health_at_level = health_multiplier;
        let base_reward_at_level = reward_multiplier;
        let base_lifetime_at_level = lifetime_multiplier;
        let base_speed = portal_config.level_scaled_stats.base_enemy_speed.calculate(portal.level);

        let max_health = base_health_at_level * enemy_config.health_coef;
        let speed = base_speed * enemy_config.speed_coef;
        let reward = base_reward_at_level * enemy_config.reward_coef;

        // Lifetime = (Base Scaled * Coef) + Bonus Independent
        let lifetime_val = (base_lifetime_at_level * enemy_config.lifetime_coef) + portal_lifetime.0.value;

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

pub fn handle_portal_upgrade(
    mut events: MessageReader<UpgradePortal>,
    mut portal_query: Query<&mut Portal>,
    mut wallet: ResMut<Wallet>,
    portal_config: Res<PortalConfig>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    for _event in events.read() {
        if let Some(mut portal) = portal_query.iter_mut().next() {
            if wallet.void_shards >= portal.upgrade_price {
                wallet.void_shards -= portal.upgrade_price;

                portal.level += 1;

                // Update Price
                match portal.price_growth_strategy {
                    GrowthStrategy::Linear => {
                        portal.upgrade_price += portal.price_growth_factor;
                    }
                    GrowthStrategy::Exponential => {
                        portal.upgrade_price *= portal.price_growth_factor;
                    }
                }

                // Update Spawn Timer (as it scales with level)
                let new_spawn_time = portal_config.level_scaled_stats.spawn_timer.calculate(portal.level);
                spawn_timer.0.set_duration(std::time::Duration::from_secs_f32(new_spawn_time));

                info!(
                    "Portal upgraded to Level {}. New Price: {}",
                    portal.level, portal.upgrade_price
                );
            } else {
                warn!("Not enough shards to upgrade portal!");
            }
        }
    }
}

pub fn handle_portal_capacity_upgrade(
    mut events: MessageReader<UpgradePortalCapacity>,
    mut portal_query: Query<&mut PortalCapacity>,
    mut wallet: ResMut<Wallet>,
) {
    for _event in events.read() {
        if let Some(mut capacity) = portal_query.iter_mut().next() {
            if wallet.void_shards >= capacity.0.price {
                wallet.void_shards -= capacity.0.price;
                capacity.0.upgrade();

                info!(
                    "Portal capacity upgraded to {}. New Price: {}",
                    capacity.0.value, capacity.0.price
                );
            } else {
                warn!("Not enough shards to upgrade portal capacity!");
            }
        }
    }
}

#[cfg(test)]
mod tests_mechanics;
