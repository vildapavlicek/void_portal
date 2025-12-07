use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

// Components
#[derive(Component)]
pub struct Portal;

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

// Resources
#[derive(Resource)]
pub struct EnemySpawnTimer(pub Timer);

// Systems

pub fn spawn_portal(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    portal_query: Query<Entity, With<Portal>>,
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
        ));
        info!("Portal spawned at y={}", portal_y);
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    enemy_query: Query<Entity, With<Enemy>>,
    portal_query: Query<&Transform, With<Portal>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        if enemy_query.iter().count() >= 5 {
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

        // Random target position within window
        let mut rng = rand::rng();
        let target_x = rng.random_range(-half_width..half_width);
        let target_y = rng.random_range(-half_height..half_height);
        let target_position = Vec2::new(target_x, target_y);

        commands.spawn((
            Sprite {
                color: Color::srgb(0.0, 0.0, 1.0), // Blue
                custom_size: Some(Vec2::new(24.0, 24.0)),
                ..default()
            },
            Transform::from_translation(portal_transform.translation),
            Enemy { target_position },
            Health {
                current: 100.0,
                max: 100.0,
            },
            Lifetime {
                timer: Timer::from_seconds(10.0, TimerMode::Once),
            },
        )).with_children(|parent| {
            parent.spawn((
                Text2d::new("100"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 20.0, 1.0)),
            ));
        });
        info!("Enemy spawned! Target: {:?}", target_position);
    }
}

pub fn despawn_dead_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Enemy>>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
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
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("{:.0}", health.current);
            }
        }
    }
}

pub fn move_enemies(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
) {
    let speed = 100.0; // Pixels per second

    for (mut transform, enemy) in enemy_query.iter_mut() {
        let direction = (enemy.target_position - transform.translation.truncate()).normalize_or_zero();
        let distance = transform.translation.truncate().distance(enemy.target_position);

        if distance > 1.0 {
            transform.translation += (direction * speed * time.delta_secs()).extend(0.0);
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
