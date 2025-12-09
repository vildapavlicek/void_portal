use {bevy::prelude::*, bevy::asset::LoadedFolder, serde::Deserialize};

#[derive(Resource, Default)]
pub struct PrefabHandles {
    pub portal: Handle<Scene>,
    pub enemies_folder: Handle<LoadedFolder>,
    pub soldier: Handle<Scene>,
}

#[derive(Asset, Clone, Debug, Deserialize, Reflect, Resource)]
pub struct PortalConfig {
    pub base_void_shards_reward: f32,
    pub base_upgrade_price: f32,
    pub upgrade_price_increase_coef: f32,
    pub base_enemy_health: f32,
    pub base_enemy_speed: f32,
    pub base_enemy_lifetime: f32,
    pub base_enemy_reward: f32,
    pub spawn_timer: f32,
}

#[derive(Asset, Clone, Debug, Deserialize, Reflect, Resource)]
pub struct EnemyConfig {
    pub spawn_limit: usize,
    pub health_coef: f32,
    pub speed_coef: f32,
    pub lifetime_coef: f32,
    pub reward_coef: f32,
    pub scene_path: String,
}

#[derive(Asset, Clone, Debug, Deserialize, Reflect, Resource)]
pub struct SoldierConfig {
    pub attack_timer: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_lifetime: f32,
}
