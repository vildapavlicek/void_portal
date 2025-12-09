use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy {
    pub target_position: Vec2,
}

#[derive(Component, Reflect, Default)]
pub struct SpawnIndex(pub u32);

#[derive(Component, Reflect)]
pub struct Reward(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PendingEnemyStats {
    pub max_health: f32,
    pub speed: f32,
    pub reward: f32,
    pub lifetime: f32,
    pub spawn_index: u32,
    pub target_position: Vec2,
}
