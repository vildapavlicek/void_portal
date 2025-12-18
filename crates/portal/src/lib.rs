#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub use common::components::*;
use {
    bevy::prelude::*,
    common::{
        ChangeActiveLevel, GameState, RequestUpgrade, SpawnMonsterRequest, UpgradePortal,
        UpgradeableStat,
    },
    monster_factory::SpawnMonsterEvent,
    monsters::{AvailableEnemies, Monster},
    rand::Rng,
    wallet::Wallet,
};

// No longer exporting local components
// mod components;
// pub use components::*;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortalSpawnTracker>();

        app.add_systems(
            Update,
            (
                layout_portal,
                (portal_tick_logic, portal_spawn_logic).chain(),
                handle_portal_upgrade,
                handle_generic_upgrades,
                handle_active_level_change,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Resources
#[derive(Resource, Default)]
pub struct PortalSpawnTracker(pub u32);

// Systems

// A. Layout
pub fn layout_portal(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut portal_query: Query<&mut Transform, (With<PortalRoot>, Changed<Transform>)>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    // Hardcoded offset as per plan/requirements
    let top_offset = 100.0;
    let portal_y = (window.height() / 2.0) - top_offset;

    for mut transform in portal_query.iter_mut() {
        if (transform.translation.y - portal_y).abs() > 0.01 {
            transform.translation.y = portal_y;
        }
    }
}

// B. Tick Logic
pub fn portal_tick_logic(
    time: Res<Time>,
    mut portal_query: Query<(Entity, &mut PortalSpawner, &Children)>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
    monster_query: Query<(), With<Monster>>,
    available_monsters: Res<AvailableEnemies>,
    mut spawn_events: MessageWriter<SpawnMonsterRequest>,
) {
    let current_monster_count = monster_query.iter().count();

    for (entity, mut spawner, children) in portal_query.iter_mut() {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() {
            if available_monsters.0.is_empty() {
                warn!("No monsters available to spawn!");
                continue;
            }

            // Iterate children to find "Capacity"
            let capacity = children
                .iter()
                .filter_map(|child| upgrade_query.get(child).ok())
                .find(|(slot, _)| slot.name == "Capacity");

            if let Some((_, cap_stat)) = capacity {
                if current_monster_count < cap_stat.value as usize {
                    spawn_events.write(SpawnMonsterRequest {
                        portal_entity: entity,
                    });
                }
            }
        }
    }
}

// C. Spawn Logic
pub fn portal_spawn_logic(
    mut events: MessageReader<SpawnMonsterRequest>,
    mut monster_events: MessageWriter<SpawnMonsterEvent>,
    portal_query: Query<(
        &Transform,
        &PortalLevel,
        &BaseMonsterHealth,
        &BaseMonsterReward,
        &BaseMonsterSpeed,
        &BaseMonsterLifetime,
        &Children,
        Option<&ScavengerPenalty>,
    )>,
    mut spawn_tracker: ResMut<PortalSpawnTracker>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if events.is_empty() {
        return;
    }

    let Some(window) = window_query.iter().next() else {
        return;
    };
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    for request in events.read() {
        // Validation query to ensure portal components exist, but hydration logic moved to monster_factory.
        if portal_query.get(request.portal_entity).is_err() {
            continue;
        };

        // Random target position calculation
        let mut rng = rand::rng();
        let target_x = rng.random_range(-half_width..half_width);
        let target_y = rng.random_range(-half_height..half_height);
        let target_position = Vec2::new(target_x, target_y);

        // Emit event with minimal data
        monster_events.write(SpawnMonsterEvent {
            asset_path: "prefabs/monsters/goblin.scn.ron".to_string(),
            portal_entity: request.portal_entity,
            spawn_index: spawn_tracker.0,
            target_position,
        });

        spawn_tracker.0 = spawn_tracker.0.wrapping_add(1);
        info!(
            "SpawnMonsterEvent emitted for Portal {:?}",
            request.portal_entity
        );
    }
}

// D. Upgrade Portal
pub fn handle_portal_upgrade(
    mut events: MessageReader<UpgradePortal>,
    mut portal_query: Query<(&mut PortalLevel, &mut UpgradeCost, &mut PortalSpawner)>,
    mut wallet: ResMut<Wallet>,
) {
    for _ in events.read() {
        if let Some((mut level, mut cost, mut spawner)) = portal_query.iter_mut().next() {
            if wallet.void_shards >= cost.current_price {
                wallet.void_shards -= cost.current_price;

                level.max_unlocked += 1;
                level.active = level.max_unlocked; // Auto-snap

                // Recalculate Price
                cost.current_price = cost.strategy.calculate(level.max_unlocked as f32);

                // Recalculate Spawn Timer
                let new_time = spawner.interval_strategy.calculate(level.active as f32);
                spawner
                    .timer
                    .set_duration(std::time::Duration::from_secs_f32(new_time));

                info!(
                    "Portal upgraded to Max Level {}. New Price: {}",
                    level.max_unlocked, cost.current_price
                );
            } else {
                warn!("Not enough shards to upgrade portal!");
            }
        }
    }
}

// E. Active Level Change
pub fn handle_active_level_change(
    mut events: MessageReader<ChangeActiveLevel>,
    mut portal_query: Query<(&mut PortalLevel, &mut PortalSpawner)>,
) {
    for event in events.read() {
        if let Ok((mut level, mut spawner)) = portal_query.get_mut(event.portal_entity) {
            let new_level =
                (level.active as i32 + event.change).clamp(1, level.max_unlocked as i32) as u32;

            if new_level != level.active {
                level.active = new_level;

                // Recalculate Spawn Timer
                let new_time = spawner.interval_strategy.calculate(level.active as f32);
                spawner
                    .timer
                    .set_duration(std::time::Duration::from_secs_f32(new_time));

                info!("Portal active level changed to {}", level.active);
            }
        }
    }
}

// F. Generic Upgrades
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
