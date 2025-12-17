#![allow(clippy::type_complexity)]

use bevy::prelude::*;

pub mod events;
pub use events::*;

pub mod stats;
pub use stats::*;

pub mod components;
pub use components::*;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Reward(pub f32);

#[derive(Component, Debug, Clone, Reflect)]
pub struct ScavengeModifier(pub f32);

#[derive(Component, Debug, Clone, Reflect)]
pub struct Dead {
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
