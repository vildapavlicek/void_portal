use bevy::prelude::*;

mod components;
mod systems;
mod tests;

pub use {components::*, systems::SpawnMonsterEvent};

pub struct MonsterFactoryPlugin;

impl Plugin for MonsterFactoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HpCoef>()
            .register_type::<SpeedCoef>()
            .register_type::<RewardCoef>()
            .register_type::<LifetimeCoef>()
            .register_type::<MonsterBuilder>();

        app.add_message::<SpawnMonsterEvent>();

        app.init_resource::<systems::PendingMonsterSpawns>();

        app.add_systems(
            Update,
            (
                systems::spawn_monster_listener,
                systems::attach_monster_context,
                systems::hydrate_monster_stats,
            ),
        );
    }
}
