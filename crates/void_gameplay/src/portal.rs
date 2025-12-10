use {
    crate::configs::{EnemyConfig, PortalConfig},
    bevy::{prelude::*, window::PrimaryWindow},
    rand::Rng,
    void_components::{Dead, Reward},
    void_core::events::{EnemyKilled, UpgradePortal},
    void_wallet::Wallet,
};

// Components
#[derive(Component)]
pub struct Portal;

#[derive(Component)]
pub struct Level(pub u32);

#[derive(Component)]
pub struct VoidShardsReward(pub f32);

#[derive(Component)]
pub struct UpgradePrice(pub f32);

#[derive(Component)]
pub struct UpgradeCoef(pub f32);

#[derive(Component)]
pub struct Enemy {
    pub target_position: Vec2,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component)]
pub struct SpawnIndex(pub u32);

#[derive(Component)]
pub struct Speed(pub f32);

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

#[derive(Resource)]
pub struct EnemySpawnTimer(pub Timer);

#[derive(Resource, Default)]
pub struct AvailableEnemies(pub Vec<EnemyConfig>);

// Systems

pub fn spawn_portal(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    portal_query: Query<Entity, With<Portal>>,
    portal_config: Res<PortalConfig>,
) {
    // Only spawn if not already spawned
    if !portal_query.is_empty() {
        return;
    }

    if let Some(window) = window_query.iter().next() {
        let half_height = window.height() / 2.0;
        let portal_y = half_height - portal_config.portal_top_offset;

        commands.spawn((
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
        ));
        info!("Portal spawned at y={}", portal_y);
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

        // For now, pick the first available enemy.
        // In the future, logic could select specific enemies.
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

        // Calculate stats with growth
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

        // Random target position within window
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

pub fn handle_dying_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Health), (With<Enemy>, Without<Dead>)>,
    mut events: MessageWriter<EnemyKilled>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands
                .entity(entity)
                .remove::<Enemy>()
                .insert(Dead {
                    despawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
                })
                .insert(Visibility::Hidden);

            events.write(EnemyKilled { entity });
            info!("Enemy died, hidden and scheduled for despawn");
        }
    }
}

pub fn despawn_dead_bodies(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dead)>,
) {
    for (entity, mut dead) in query.iter_mut() {
        dead.despawn_timer.tick(time.delta());
        if dead.despawn_timer.is_finished() {
            commands.entity(entity).despawn();
            info!("Dead enemy body despawned");
        }
    }
}

pub fn update_enemy_health_ui(
    enemy_query: Query<(&Health, &Children), (With<Enemy>, Changed<Health>)>,
    mut text_query: Query<&mut Text2d>,
) {
    for (health, children) in enemy_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("{:.0}", health.current);
            }
        }
    }
}

pub fn move_enemies(time: Res<Time>, mut enemy_query: Query<(&mut Transform, &Enemy, &Speed)>) {
    for (mut transform, enemy, speed) in enemy_query.iter_mut() {
        let direction =
            (enemy.target_position - transform.translation.truncate()).normalize_or_zero();
        let distance = transform
            .translation
            .truncate()
            .distance(enemy.target_position);

        if distance > 1.0 {
            transform.translation += (direction * speed.0 * time.delta_secs()).extend(0.0);
        }
    }
}

pub fn enemy_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetime_query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in lifetime_query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
            info!("Enemy despawned due to lifetime expiry");
        }
    }
}
