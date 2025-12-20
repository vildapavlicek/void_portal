#![allow(clippy::type_complexity)]

use bevy::prelude::*;

pub mod messages;
pub use messages::*;

pub mod stats;
pub use stats::*;

pub mod components;
pub use components::*;

pub mod requirements;
pub use requirements::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        // Messages
        app.add_message::<SpawnMonsterRequest>()
            .add_message::<MonsterKilled>()
            .add_message::<UpgradePortal>()
            .add_message::<RequestUpgrade>()
            .add_message::<ChangeActiveLevel>()
            .add_message::<MonsterScavenged>()
            .add_message::<DamageMessage>()
            .add_message::<MeleeHitMessage>()
            .add_message::<ProjectileCollisionMessage>()
            .add_message::<SpawnFloatingText>();

        // Components & Types
        app
            // components.rs
            .register_type::<PortalRoot>()
            .register_type::<PortalLevel>()
            .register_type::<UpgradeCost>()
            .register_type::<PortalSpawner>()
            .register_type::<BaseMonsterHealth>()
            .register_type::<BaseMonsterReward>()
            .register_type::<BaseMonsterSpeed>()
            .register_type::<BaseMonsterLifetime>()
            .register_type::<BaseMonsterArmor>()
            .register_type::<ScavengerPenalty>()
            .register_type::<UpgradeSlot>()
            .register_type::<PortalUpgrades>()
            .register_type::<LockedFeature>()
            // lib.rs
            .register_type::<Reward>()
            .register_type::<ScavengeModifier>()
            .register_type::<MarkedForCleanUp>()
            // stats.rs
            .register_type::<GrowthStrategy>()
            .register_type::<ConditionalUpgrade>()
            .register_type::<UpgradeableStat>()
            .register_type::<MeleeDamageContext>()
            .register_type::<ProjectileDamageContext>()
            // requirements.rs
            .register_type::<Requirement>();
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Reward(pub f32);

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ScavengeModifier(pub f32);

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MarkedForCleanUp {
    pub despawn_timer: Timer,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum VoidGameStage {
    /// Run as first state on new frame. Things that need to be processed
    /// before we resolve intents and actions should be processed here
    FrameStart,
    /// Resolve intent: Resolves what the given entity will do at that frame (attack, move, idle, etc).
    ResolveIntent,
    /// Actions: Processes the intent (e.g., AttackIntent -> AttackAction).
    Actions,
    /// Effect: Applies effects (e.g., AttackAction -> DealDamageEffect).
    Effect,
    /// Frame-end: Maintenance stage (despawn dead, process rewards, etc.).
    FrameEnd,
}
