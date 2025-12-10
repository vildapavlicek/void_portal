use {
    bevy::{prelude::*, window::PrimaryWindow},
    bevy_common_assets::ron::RonAssetPlugin,
    common::{GameState, Reward, UpgradePortal},
    enemy::{AvailableEnemies, Enemy, Health, Lifetime, SpawnIndex, Speed},
    rand::Rng,
    serde::Deserialize,
    wallet::Wallet,
};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<PortalConfig>::new(&["portal.ron"]));

        app.register_type::<Portal>()
            .register_type::<Level>()
            .register_type::<VoidShardsReward>()
            .register_type::<UpgradePrice>()
            .register_type::<UpgradeCoef>()
            .register_type::<PortalConfig>();

        app.init_resource::<PortalSpawnTracker>()
            .init_resource::<EnemySpawnTimer>();

        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_portal, init_enemy_spawn_timer),
        );

        app.add_systems(
            Update,
            (spawn_enemies, handle_portal_upgrade).run_if(in_state(GameState::Playing)),
        );
    }
}

// Configs
#[derive(Deserialize, Asset, Clone, Debug, Resource, Reflect)]
pub struct PortalConfig {
    pub spawn_timer: f32,
    pub base_void_shards_reward: f32,
    pub base_upgrade_price: f32,
    pub upgrade_price_increase_coef: f32,
    pub portal_top_offset: f32,
    // Base enemy stats
    pub base_enemy_health: f32,
    pub base_enemy_speed: f32,
    pub base_enemy_lifetime: f32,
    pub base_enemy_reward: f32,
    // Growth
    pub enemy_health_growth_factor: f32,
    pub enemy_reward_growth_factor: f32,
}

// Components
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Portal;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Level(pub u32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct VoidShardsReward(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct UpgradePrice(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct UpgradeCoef(pub f32);

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

#[derive(Resource, Default)]
pub struct EnemySpawnTimer(pub Timer);

// Systems
fn init_enemy_spawn_timer(mut commands: Commands, portal_config: Res<PortalConfig>) {
    commands.insert_resource(EnemySpawnTimer(Timer::from_seconds(
        portal_config.spawn_timer,
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

        let entity = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.5, 0.0, 0.5), // Purple
                    custom_size: Some(Vec2::new(16.0, 32.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, portal_y, 0.0),
                Portal,
                Level(1),
                VoidShardsReward(portal_config.base_void_shards_reward),
                UpgradePrice(portal_config.base_upgrade_price),
                UpgradeCoef(portal_config.upgrade_price_increase_coef),
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
    portal_query: Query<(&Transform, &Level), With<Portal>>,
    mut spawn_tracker: ResMut<PortalSpawnTracker>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        if available_enemies.0.is_empty() {
            warn!("No enemies available to spawn!");
            return;
        }

        let enemy_config = &available_enemies.0[0];

        if enemy_query.iter().count() >= enemy_config.spawn_limit {
            info!("Max enemies reached, skipping spawn");
            return;
        }

        let Some((portal_transform, portal_level)) = portal_query.iter().next() else {
            warn!("No portal found to spawn enemies from");
            return;
        };

        let Some(window) = window_query.iter().next() else {
            return;
        };

        let half_width = window.width() / 2.0;
        let half_height = window.height() / 2.0;

        let level_exponent = (portal_level.0 as f32) - 1.0;
        let health_multiplier = portal_config
            .enemy_health_growth_factor
            .powf(level_exponent);
        let reward_multiplier = portal_config
            .enemy_reward_growth_factor
            .powf(level_exponent);

        let max_health =
            (portal_config.base_enemy_health * enemy_config.health_coef) * health_multiplier;
        let speed = portal_config.base_enemy_speed * enemy_config.speed_coef;
        let lifetime = portal_config.base_enemy_lifetime * enemy_config.lifetime_coef;
        let reward =
            (portal_config.base_enemy_reward * enemy_config.reward_coef) * reward_multiplier;

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
                    timer: Timer::from_seconds(lifetime, TimerMode::Once),
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
    mut portal_query: Query<(&mut Level, &mut UpgradePrice, &UpgradeCoef), With<Portal>>,
    mut wallet: ResMut<Wallet>,
) {
    for _event in events.read() {
        if let Ok((mut level, mut upgrade_price, upgrade_coef)) = portal_query.single_mut() {
            if wallet.void_shards >= upgrade_price.0 {
                wallet.void_shards -= upgrade_price.0;
                level.0 += 1;
                upgrade_price.0 *= upgrade_coef.0;

                info!(
                    "Portal upgraded to Level {}. New Price: {}",
                    level.0, upgrade_price.0
                );
            } else {
                warn!("Not enough shards to upgrade portal!");
            }
        }
    }
}

#[cfg(test)]
mod tests_mechanics;
