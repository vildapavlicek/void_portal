use {bevy::prelude::*, serde::Deserialize};

#[derive(Deserialize, Asset, TypePath, Clone, Debug, Resource)]
pub struct GlobalConfig {
    pub spawn_timer: f32,
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug, Resource)]
pub struct EnemyConfig {
    pub max_health: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub spawn_limit: usize,
    pub reward: f32,
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug, Resource)]
pub struct SoldierConfig {
    pub attack_timer: f32,
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_lifetime: f32,
    pub attack_range: f32,
    pub move_speed: f32,
}
