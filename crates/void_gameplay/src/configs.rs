use {bevy::prelude::*, serde::Deserialize};

#[derive(Deserialize, Asset, TypePath, Clone, Debug, Resource)]
pub struct PortalConfig {
    pub spawn_timer: f32,
    pub base_void_shards_reward: f32,
    pub base_upgrade_price: f32,
    pub upgrade_price_increase_coef: f32,
    pub portal_top_offset: f32,
    // Base enemy stats
    pub base_enemy_health: f32,
    pub base_enemy_speed: f32,
    pub base_enemy_lifetime: f32,
    pub base_enemy_reward: f32,
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug, Resource)]
pub struct EnemyConfig {
    pub health_coef: f32,
    pub lifetime_coef: f32,
    pub speed_coef: f32,
    pub reward_coef: f32,
    pub spawn_limit: usize,
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
