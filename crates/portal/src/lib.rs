#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use {
    bevy::{prelude::*, window::PrimaryWindow},
    bevy_common_assets::ron::RonAssetPlugin,
    common::{GameState, GrowthStrategy, RequestUpgrade, Reward, UpgradePortal, UpgradeableStat},
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
            .register_type::<UpgradeSlot>()
            .register_type::<UpgradeableStat>()
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
                handle_generic_upgrades,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Components
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Portal {
    pub level: u32,
    pub upgrade_price: f32,
    pub price_growth_factor: f32,
    pub price_growth_strategy: GrowthStrategy,
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct UpgradeSlot {
    pub name: String,
}

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

        let portal_entity = commands
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
                Pickable::default(),
            ))
            .id();

        // Spawn upgrades as children
        commands.entity(portal_entity).with_children(|parent| {
            for (name, config) in &portal_config.upgrades {
                let stat = UpgradeableStat::new(
                    config.value,
                    config.price,
                    config.growth_factor,
                    config.growth_strategy,
                    config.price_growth_factor,
                    config.price_growth_strategy,
                );

                parent.spawn((UpgradeSlot { name: name.clone() }, stat));
            }
        });

        info!(
            "Portal spawned at y={} | entity={portal_entity:?}",
            portal_y
        );
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    portal_config: Res<PortalConfig>,
    available_enemies: Res<AvailableEnemies>,
    enemy_query: Query<Entity, With<Enemy>>,
    portal_query: Query<(&Transform, &Portal, &Children)>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
    mut spawn_tracker: ResMut<PortalSpawnTracker>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        if available_enemies.0.is_empty() {
            warn!("No enemies available to spawn!");
            return;
        }

        let Some((portal_transform, portal, children)) = portal_query.iter().next() else {
            warn!("No portal found to spawn enemies from");
            return;
        };

        // Find Capacity and Lifetime upgrades
        let mut capacity_val = None;
        let mut lifetime_bonus = None;

        for &child in children {
            if let Ok((slot, stat)) = upgrade_query.get(child) {
                if slot.name == "Capacity" {
                    capacity_val = Some(stat.value);
                } else if slot.name == "Lifetime" {
                    lifetime_bonus = Some(stat.value);
                }
            }
        }

        let capacity_val = capacity_val.expect("Capacity upgrade not found on Portal children!");
        let lifetime_bonus =
            lifetime_bonus.expect("Lifetime upgrade not found on Portal children!");

        let enemy_config = &available_enemies.0[0];

        if enemy_query.iter().count() >= capacity_val as usize {
            info!("Max enemies reached, skipping spawn");
            return;
        }

        let Some(window) = window_query.iter().next() else {
            return;
        };

        let half_width = window.width() / 2.0;
        let half_height = window.height() / 2.0;

        // Dynamic stats calculation based on Portal Level
        let health_multiplier = portal_config
            .level_scaled_stats
            .enemy_health
            .calculate(portal.level);
        let reward_multiplier = portal_config
            .level_scaled_stats
            .void_shards_reward
            .calculate(portal.level);
        let lifetime_multiplier = portal_config
            .level_scaled_stats
            .base_enemy_lifetime
            .calculate(portal.level);

        let base_health_at_level = health_multiplier;
        let base_reward_at_level = reward_multiplier;
        let base_lifetime_at_level = lifetime_multiplier;
        let base_speed = portal_config
            .level_scaled_stats
            .base_enemy_speed
            .calculate(portal.level);

        let max_health = base_health_at_level * enemy_config.health_coef;
        let speed = base_speed * enemy_config.speed_coef;
        let reward = base_reward_at_level * enemy_config.reward_coef;

        // Lifetime = (Base Scaled * Coef) + Bonus Independent
        let lifetime_val = (base_lifetime_at_level * enemy_config.lifetime_coef) + lifetime_bonus;

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
                let new_spawn_time = portal_config
                    .level_scaled_stats
                    .spawn_timer
                    .calculate(portal.level);
                spawn_timer
                    .0
                    .set_duration(std::time::Duration::from_secs_f32(new_spawn_time));

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
