#![allow(clippy::type_complexity)]
use {
    crate::components::*,
    bevy::prelude::*,
    common::{Reward, ScavengeModifier},
    enemy::{Enemy, Health, Lifetime, SpawnIndex, Speed},
    std::collections::HashMap,
};

/// Trigger to spawn a monster
#[derive(Message, Clone)]
pub struct SpawnMonsterEvent {
    pub asset_path: String,
    pub context: MonsterSpawnContext,
}

/// Resource to track pending spawns from the SceneSpawner
#[derive(Resource, Default)]
pub struct PendingMonsterSpawns(HashMap<bevy::scene::InstanceId, MonsterSpawnContext>);

/// 1. The Listener: Starts the spawn process
pub fn spawn_monster_listener(
    mut events: MessageReader<SpawnMonsterEvent>,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
    mut pending_spawns: ResMut<PendingMonsterSpawns>,
) {
    for event in events.read() {
        let scene_handle = asset_server.load(&event.asset_path);
        let instance_id = scene_spawner.spawn_dynamic(scene_handle);
        pending_spawns.0.insert(instance_id, event.context);
    }
}

/// 2. The Attacher: Waits for instances to be ready and attaches context/transform
pub fn attach_monster_context(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    mut pending_spawns: ResMut<PendingMonsterSpawns>,
) {
    let mut to_remove = Vec::new();

    for (instance_id, context) in pending_spawns.0.iter() {
        if scene_spawner.instance_is_ready(*instance_id) {
            let entities: Vec<Entity> =
                scene_spawner.iter_instance_entities(*instance_id).collect();

            for entity in entities {
                commands
                    .entity(entity)
                    .insert(*context)
                    .insert(Transform::from_translation(
                        context.spawn_position.extend(0.0),
                    ))
                    // Ensure visibility is correct if scene defaults are weird
                    .insert(Visibility::Inherited);
            }
            to_remove.push(*instance_id);
        }
    }

    for id in to_remove {
        pending_spawns.0.remove(&id);
    }
}

/// 3. The Hydrator: Reacts to entities with Context and Coefs
pub fn hydrate_monster_stats(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &MonsterSpawnContext,
            Option<&HpCoef>,
            Option<&SpeedCoef>,
            Option<&RewardCoef>,
            Option<&LifetimeCoef>,
        ),
        (Without<Health>, With<HpCoef>),
    >,
) {
    for (entity, context, hp_coef, speed_coef, reward_coef, lifetime_coef) in query.iter() {
        let mut entity_cmds = commands.entity(entity);

        // 1. Health
        if let Some(coef) = hp_coef {
            let final_hp = context.base_health * coef.val;
            entity_cmds.insert(Health {
                current: final_hp,
                max: final_hp,
            });
            entity_cmds.remove::<HpCoef>();
        }

        // 2. Speed
        if let Some(coef) = speed_coef {
            let final_speed = context.base_speed * coef.val;
            entity_cmds.insert(Speed(final_speed));
            entity_cmds.remove::<SpeedCoef>();
        }

        // 3. Reward
        if let Some(coef) = reward_coef {
            let final_reward = context.base_reward * coef.val;
            entity_cmds.insert(Reward(final_reward));
            entity_cmds.remove::<RewardCoef>();
        }

        // 4. Lifetime
        if let Some(coef) = lifetime_coef {
            let final_lifetime = (context.base_lifetime * coef.val) + context.bonus_lifetime;
            entity_cmds.insert(Lifetime {
                timer: Timer::from_seconds(final_lifetime, TimerMode::Once),
            });
            entity_cmds.remove::<LifetimeCoef>();
        }

        // 5. Scavenge Modifier
        entity_cmds.insert(ScavengeModifier(context.scavenger_penalty));

        // 6. Spawn Index & Enemy Marker
        entity_cmds.insert(SpawnIndex(context.spawn_index));
        entity_cmds.insert(Enemy {
            target_position: context.target_position,
        });

        // 7. Cleanup
        entity_cmds.remove::<MonsterSpawnContext>();
    }
}
