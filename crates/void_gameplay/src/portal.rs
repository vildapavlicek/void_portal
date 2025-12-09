use {
    crate::configs::{EnemyConfig, PortalConfig},
    bevy::{prelude::*, window::PrimaryWindow},
    rand::Rng,
    void_core::events::EnemyKilled,
};

// Components
#[derive(Component, Reflect)]
pub struct Portal;

#[derive(Component, Reflect)]
pub struct VoidShardsReward(pub f32);

#[derive(Component, Reflect)]
pub struct UpgradePrice(pub f32);

#[derive(Component, Reflect)]
pub struct UpgradeCoef(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct EnemySpawner {
    pub timer: Timer,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy {
    pub target_position: Vec2,
}

#[derive(Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Reflect)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component, Reflect, Default)]
pub struct SpawnIndex(pub u32);

#[derive(Component, Reflect)]
pub struct Reward(pub f32);

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PendingEnemyStats {
    pub max_health: f32,
    pub speed: f32,
    pub reward: f32,
    pub lifetime: f32,
    pub spawn_index: u32,
    pub target_position: Vec2,
}

#[derive(Clone)]
pub struct LoadedEnemy {
    pub config: EnemyConfig,
    pub scene: Handle<Scene>,
}

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

#[derive(Resource)]
pub struct EnemySpawnTimer(pub Timer);

#[derive(Resource, Default)]
pub struct AvailableEnemies(pub Vec<LoadedEnemy>);

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
        let portal_y = half_height - 50.0; // Fixed offset from top

        commands.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.0, 0.5), // Purple
                custom_size: Some(Vec2::new(16.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(0.0, portal_y, 0.0),
            Portal,
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
    portal_query: Query<&Transform, With<Portal>>,
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
        let loaded_enemy = &available_enemies.0[0];
        let enemy_config = &loaded_enemy.config;

        if enemy_query.iter().count() >= enemy_config.spawn_limit {
            info!("Max enemies reached, skipping spawn");
            return;
        }

        let Some(portal_transform) = portal_query.iter().next() else {
            warn!("No portal found to spawn enemies from");
            return;
        };

        let Some(window) = window_query.iter().next() else {
            return;
        };

        let half_width = window.width() / 2.0;
        let half_height = window.height() / 2.0;

        // Calculate stats
        let max_health = portal_config.base_enemy_health * enemy_config.health_coef;
        let speed = portal_config.base_enemy_speed * enemy_config.speed_coef;
        let lifetime = portal_config.base_enemy_lifetime * enemy_config.lifetime_coef;
        let reward = portal_config.base_enemy_reward * enemy_config.reward_coef;

        // Random target position within window
        let mut rng = rand::rng();
        let target_x = rng.random_range(-half_width..half_width);
        let target_y = rng.random_range(-half_height..half_height);
        let target_position = Vec2::new(target_x, target_y);

        commands
            .spawn((
                SceneRoot(loaded_enemy.scene.clone()),
                Transform::from_translation(portal_transform.translation),
                PendingEnemyStats {
                    max_health,
                    speed,
                    reward,
                    lifetime,
                    spawn_index: spawn_tracker.0,
                    target_position,
                },
            ));

        spawn_tracker.0 = spawn_tracker.0.wrapping_add(1);
        info!("Enemy spawned via scene! Target: {:?}", target_position);
    }
}

pub fn on_enemy_spawned(
    trigger: On<Add, Enemy>,
    mut commands: Commands,
    query: Query<(Entity, Option<&PendingEnemyStats>, Option<&Children>)>,
    mut text_query: Query<&mut Text2d>,
) {
    // The trigger entity is the one that just got the Enemy component.
    // In our scene structure, the Enemy component is on the root entity.
    let entity = trigger.entity;

    if let Ok((_ent, pending_stats_opt, children_opt)) = query.get(entity) {
         if let Some(stats) = pending_stats_opt {
             commands.entity(entity).insert((
                 Health {
                     current: stats.max_health,
                     max: stats.max_health,
                 },
                 Lifetime {
                     timer: Timer::from_seconds(stats.lifetime, TimerMode::Once),
                 },
                 Reward(stats.reward),
                 Speed(stats.speed),
                 SpawnIndex(stats.spawn_index),
                 // Update the target position in the Enemy component (which was just added from scene)
                 Enemy { target_position: stats.target_position },
             ));

             // Update children text
             if let Some(children) = children_opt {
                 for child in children.iter() {
                     // Removed * dereference
                     if let Ok(mut text) = text_query.get_mut(child) {
                         text.0 = format!("{:.0}", stats.max_health);
                     }
                 }
             }

             // Remove pending stats
             commands.entity(entity).remove::<PendingEnemyStats>();

             info!("Enemy stats initialized from PendingEnemyStats");
         } else {
             // If PendingEnemyStats is missing, maybe it's on the parent?
             // But for SceneRoot, the components are added to the entity itself.
             // warn!("Enemy spawned but PendingEnemyStats not found on entity {:?}", entity);
         }
    }
}

pub fn despawn_dead_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Reward), With<Enemy>>,
    mut events: MessageWriter<EnemyKilled>,
) {
    for (entity, health, reward) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
            events.write(EnemyKilled { reward: reward.0 });
            info!("Enemy despawned due to being killed");
        }
    }
}

pub fn update_enemy_health_ui(
    enemy_query: Query<(&Health, &Children), (With<Enemy>, Changed<Health>)>,
    mut text_query: Query<&mut Text2d>,
) {
    for (health, children) in enemy_query.iter() {
        for child in children.iter() {
            // Removed * dereference
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("{:.0}", health.current);
            }
        }
    }
}

pub fn move_enemies(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &Enemy, &Speed)>,
) {
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
