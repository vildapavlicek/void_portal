use bevy::prelude::*;

/// Added to the entity at spawn time to carry context for hydration.
/// Removed after hydration is complete.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct MonsterSpawnContext {
    pub base_health: f32,
    pub base_speed: f32,
    pub base_reward: f32,
    pub base_lifetime: f32,
    pub bonus_lifetime: f32, // From portal upgrades
    pub scavenger_penalty: f32,
    pub spawn_index: u32,
    pub portal_level: u32,
    pub target_position: Vec2,
    pub spawn_position: Vec2, // Where the monster starts
}

impl Default for MonsterSpawnContext {
    fn default() -> Self {
        Self {
            base_health: 10.0,
            base_speed: 50.0,
            base_reward: 1.0,
            base_lifetime: 10.0,
            bonus_lifetime: 0.0,
            scavenger_penalty: 1.0,
            spawn_index: 0,
            portal_level: 1,
            target_position: Vec2::ZERO,
            spawn_position: Vec2::ZERO,
        }
    }
}

// --- Coefficient Proxies (Loaded from Scene) ---

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HpCoef {
    pub val: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpeedCoef {
    pub val: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RewardCoef {
    pub val: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct LifetimeCoef {
    pub val: f32,
}
