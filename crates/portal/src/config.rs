use {bevy::prelude::*, common::GrowthStrategy, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Asset, Clone, Debug, Resource, Reflect)]
pub struct PortalConfig {
    pub level: u32,
    pub level_up_price: GrowthStrategy,
    pub portal_top_offset: f32,
    pub level_scaled_stats: LevelScaledStats,
    pub upgrades: HashMap<String, IndependentStatConfig>,
}

#[derive(Deserialize, Clone, Debug, Reflect, Default)]
pub struct LevelScaledStats {
    pub void_shards_reward: GrowthStrategy,
    pub spawn_timer: GrowthStrategy,
    pub enemy_health: GrowthStrategy,
    pub base_enemy_speed: GrowthStrategy,
    pub base_enemy_lifetime: GrowthStrategy,
}

#[derive(Deserialize, Clone, Debug, Reflect)]
pub struct IndependentStatConfig {
    pub value: GrowthStrategy,
    pub price: GrowthStrategy,
}
