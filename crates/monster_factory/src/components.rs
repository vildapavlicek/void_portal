use bevy::prelude::*;

/// Added to the entity at spawn time to carry reference to creator.
/// Removed after hydration is complete.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct MonsterBuilder {
    pub portal_entity: Entity,
    pub spawn_index: u32,
    pub target_position: Vec2,
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
