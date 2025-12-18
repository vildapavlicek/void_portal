#![allow(clippy::type_complexity)]
use {
    crate::components::*,
    bevy::prelude::*,
    common::{
        components::{
            BaseMonsterHealth, BaseMonsterLifetime, BaseMonsterReward, BaseMonsterSpeed,
            PortalLevel, PortalRoot, ScavengerPenalty,
        },
        Reward, ScavengeModifier, UpgradeSlot, UpgradeableStat,
    },
    monsters::{Health, Lifetime, Monster, SpawnIndex, Speed},
    std::collections::HashMap,
};

/// Trigger to spawn a monster
#[derive(Message, Clone)]
pub struct SpawnMonsterEvent {
    pub asset_path: String,
    pub portal_entity: Entity,
    pub spawn_index: u32,
    pub target_position: Vec2,
}

/// Resource to track pending spawns from the SceneSpawner
#[derive(Resource, Default)]
pub struct PendingMonsterSpawns(HashMap<bevy::scene::InstanceId, MonsterBuilder>);

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
        let builder = MonsterBuilder {
            portal_entity: event.portal_entity,
            spawn_index: event.spawn_index,
            target_position: event.target_position,
        };
        pending_spawns.0.insert(instance_id, builder);
    }
}

/// 2. The Attacher: Waits for instances to be ready and attaches builder/transform
pub fn attach_monster_context(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    mut pending_spawns: ResMut<PendingMonsterSpawns>,
    portal_query: Query<&Transform, With<PortalRoot>>,
) {
    let mut to_remove = Vec::new();

    for (instance_id, builder) in pending_spawns.0.iter() {
        if scene_spawner.instance_is_ready(*instance_id) {
            let entities: Vec<Entity> =
                scene_spawner.iter_instance_entities(*instance_id).collect();

            // Determine spawn position from portal
            let spawn_translation = if let Ok(portal_tf) = portal_query.get(builder.portal_entity) {
                portal_tf.translation
            } else {
                Vec3::ZERO
            };

            // Only parent needs updating of the transform
            // as children's transform should be offset from the parent's
            if let Some(entity) = entities.first() {
                commands
                    .entity(*entity)
                    .insert(*builder)
                    .insert(Transform::from_translation(spawn_translation));
            }

            for entity in entities {
                commands
                    .entity(entity)
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

/// 3. The Hydrator: Reacts to entities with Builder and Coefs
pub fn hydrate_monster_stats(
    mut commands: Commands,
    monster_query: Query<(
        Entity,
        &MonsterBuilder,
        Option<&HpCoef>,
        Option<&SpeedCoef>,
        Option<&RewardCoef>,
        Option<&LifetimeCoef>,
    )>,
    // Query components from the Portal (Source of Truth)
    portal_query: Query<
        (
            &PortalLevel,
            &BaseMonsterHealth,
            &BaseMonsterReward,
            &BaseMonsterSpeed,
            &BaseMonsterLifetime,
            &Children,
            Option<&ScavengerPenalty>,
        ),
        With<PortalRoot>,
    >,
    // Query generic stats for the "Lifetime" upgrade
    upgrade_stat_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
) {
    for (entity, builder, hp_coef, speed_coef, reward_coef, lifetime_coef) in monster_query.iter() {
        let mut entity_cmds = commands.entity(entity);

        // 1. Fetch Portal Data
        let Ok((
            level,
            health_scaling,
            reward_scaling,
            speed_scaling,
            lifetime_scaling,
            children,
            scav_penalty_opt,
        )) = portal_query.get(builder.portal_entity)
        else {
            warn!(
                "Portal entity {:?} missing for monster hydration",
                builder.portal_entity
            );
            // If portal is gone, maybe just despawn the pending monster or spawn with defaults?
            // Despawning is safer to avoid zombie state.
            entity_cmds.despawn();
            continue;
        };

        let scavenger_penalty = scav_penalty_opt.map(|p| p.0).unwrap_or(1.0);

        let bonus_lifetime = children
            .iter()
            .filter_map(|child| upgrade_stat_query.get(child).ok())
            .find_map(|(slot, stat)| (slot.name == "Lifetime").then_some(stat.value))
            .unwrap_or_default();

        // 3. Calculate Base Stats
        let base_health = health_scaling.0.calculate(level.active as f32);
        let base_speed = speed_scaling.0.calculate(level.active as f32);
        let base_reward = reward_scaling.0.calculate(level.active as f32);
        let base_lifetime = lifetime_scaling.0.calculate(level.active as f32);

        // 4. Apply Coefficients and Insert Components

        // Health
        let final_hp = if let Some(coef) = hp_coef {
            base_health * coef.val
        } else {
            base_health
        };
        entity_cmds.insert(Health {
            current: final_hp,
            max: final_hp,
        });
        entity_cmds.remove::<HpCoef>();

        // Speed
        let final_speed = if let Some(coef) = speed_coef {
            base_speed * coef.val
        } else {
            base_speed
        };
        entity_cmds.insert(Speed(final_speed));
        entity_cmds.remove::<SpeedCoef>();

        // Reward
        let final_reward = if let Some(coef) = reward_coef {
            base_reward * coef.val
        } else {
            base_reward
        };
        entity_cmds.insert(Reward(final_reward));
        entity_cmds.remove::<RewardCoef>();

        // Lifetime
        let final_lifetime = if let Some(coef) = lifetime_coef {
            (base_lifetime * coef.val) + bonus_lifetime
        } else {
            base_lifetime + bonus_lifetime
        };

        entity_cmds.insert(Lifetime {
            timer: Timer::from_seconds(final_lifetime, TimerMode::Once),
        });
        entity_cmds.remove::<LifetimeCoef>();

        // Scavenge Modifier
        entity_cmds.insert(ScavengeModifier(scavenger_penalty));

        // Spawn Index & Monster Marker
        entity_cmds.insert(SpawnIndex(builder.spawn_index));
        entity_cmds.insert(Monster {
            target_position: builder.target_position,
        });

        // Cleanup
        entity_cmds.remove::<MonsterBuilder>();
    }
}
