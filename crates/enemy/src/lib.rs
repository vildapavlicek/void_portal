#![allow(clippy::type_complexity)]

use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    common::{Dead, EnemyKilled, GameState},
    serde::Deserialize,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<EnemyConfig>::new(&["enemy.ron"]));

        app.register_type::<Enemy>()
            .register_type::<Health>()
            .register_type::<Lifetime>()
            .register_type::<SpawnIndex>()
            .register_type::<Speed>()
            .register_type::<EnemyConfig>();

        app.init_resource::<AvailableEnemies>();

        app.add_systems(
            Update,
            (
                move_enemies,
                enemy_lifetime,
                handle_dying_enemies,
                despawn_dead_bodies,
                update_enemy_health_ui,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Configs
#[derive(Deserialize, Asset, Clone, Debug, Resource, Reflect)]
pub struct EnemyConfig {
    pub health_coef: f32,
    pub lifetime_coef: f32,
    pub speed_coef: f32,
    pub reward_coef: f32,
    pub spawn_time_coef: f32,
}

#[derive(Resource, Default)]
pub struct AvailableEnemies(pub Vec<EnemyConfig>);

// Components
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy {
    pub target_position: Vec2,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpawnIndex(pub u32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Speed(pub f32);

// Systems
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

#[cfg(test)]
mod tests_lifecycle;
