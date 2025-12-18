#![allow(clippy::type_complexity)]

use {
    bevy::prelude::*,
    bevy_common_assets::ron::RonAssetPlugin,
    common::{
        messages::DamageMessage, Dead, GameState, MonsterKilled, MonsterScavenged, Reward,
        ScavengeModifier, VoidGameStage,
    },
    serde::Deserialize,
};

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<MonsterConfig>::new(&["monster.ron"]));

        app.register_type::<Monster>()
            .register_type::<Health>()
            .register_type::<Lifetime>()
            .register_type::<SpawnIndex>()
            .register_type::<Speed>()
            .register_type::<MonsterConfig>()
            .register_type::<LifetimeText>();

        app.init_resource::<AvailableEnemies>();

        app.add_systems(
            Update,
            (
                move_monsters.in_set(VoidGameStage::Actions),
                apply_damage_logic.in_set(VoidGameStage::Effect),
                (
                    manage_monster_lifecycle,
                    despawn_dead_bodies,
                    update_monster_health_ui,
                    update_lifetime_text,
                )
                    .in_set(VoidGameStage::FrameEnd),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Configs
#[derive(Deserialize, Asset, Clone, Debug, Resource, Reflect)]
pub struct MonsterConfig {
    pub health_coef: f32,
    pub lifetime_coef: f32,
    pub speed_coef: f32,
    pub reward_coef: f32,
}

#[derive(Resource, Default)]
pub struct AvailableEnemies(pub Vec<MonsterConfig>);

// Components
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Monster {
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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct LifetimeText;

// Systems
pub fn move_monsters(
    time: Res<Time>,
    mut monster_query: Query<(&mut Transform, &Monster, &Speed), Without<Dead>>,
) {
    for (mut transform, monster, speed) in monster_query.iter_mut() {
        let direction =
            (monster.target_position - transform.translation.truncate()).normalize_or_zero();
        let distance = transform
            .translation
            .truncate()
            .distance(monster.target_position);

        if distance > 1.0 {
            transform.translation += (direction * speed.0 * time.delta_secs()).extend(0.0);
        }
    }
}

pub fn apply_damage_logic(
    mut messages: MessageReader<DamageMessage>,
    mut monster_query: Query<(Entity, &mut Health), With<Monster>>,
) {
    for msg in messages.read() {
        if let Ok((entity, mut health)) = monster_query.get_mut(msg.target) {
            health.current -= msg.amount;
            debug!("Unit {:?} took {} damage", entity, msg.amount);
        }
    }
}
pub fn manage_monster_lifecycle(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Lifetime,
            &Health,
            &Reward,
            &Transform,
            Option<&ScavengeModifier>,
        ),
        (With<Monster>, Without<Dead>),
    >,
    mut kill_events: MessageWriter<MonsterKilled>,
    mut scavenge_events: MessageWriter<MonsterScavenged>,
) {
    for (entity, mut lifetime, health, reward, transform, modifier) in query.iter_mut() {
        // 1. Priority Check: Is the monster dead?
        if health.current <= 0.0 {
            commands
                .entity(entity)
                .remove::<Monster>()
                .insert(Dead {
                    despawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
                })
                .insert(Visibility::Hidden);

            kill_events.write(MonsterKilled { entity });
            info!("Monster died, hidden and scheduled for despawn");

            // Critical: Continue to next entity so we don't process lifetime for a dead unit
            continue;
        }

        // 2. Secondary Check: Has lifetime expired?
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            // Scavenger Logic (only if unit wasn't killed)
            let damage_dealt = health.max - health.current;
            if damage_dealt > 0.0 {
                let percentage = damage_dealt / health.max;
                let penalty = modifier.map(|m| m.0).unwrap_or(0.0);
                let amount = reward.0 * percentage * penalty;

                if amount > 0.0 {
                    scavenge_events.write(MonsterScavenged {
                        amount,
                        location: transform.translation,
                    });
                    info!("Monster scavenged for {}", amount);
                }
            }

            commands.entity(entity).despawn();
            info!("Monster despawned due to lifetime expiry");
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
            info!("Dead monster body despawned");
        }
    }
}

pub fn update_monster_health_ui(
    monster_query: Query<(&Health, &Children), (With<Monster>, Changed<Health>)>,
    mut text_query: Query<&mut Text2d, Without<LifetimeText>>,
) {
    for (health, children) in monster_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("{:.0}", health.current);
            }
        }
    }
}

pub fn update_lifetime_text(
    monster_query: Query<(&Lifetime, &Children), With<Monster>>,
    mut text_query: Query<&mut Text2d, With<LifetimeText>>,
) {
    for (lifetime, children) in monster_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("{:.1}s", lifetime.timer.remaining_secs());
            }
        }
    }
}

#[cfg(test)]
mod tests_lifecycle;
