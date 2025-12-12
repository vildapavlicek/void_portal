use {bevy::prelude::*, common::GrowthStrategy, serde::Deserialize, std::collections::HashMap};

// Configs
#[derive(Deserialize, Asset, Clone, Debug, Resource, Reflect)]
pub struct PortalConfig {
    pub level: u32,
    pub level_up_price: LevelUpConfig,
    pub portal_top_offset: f32,
    pub level_scaled_stats: LevelScaledStats,
    pub upgrades: HashMap<String, IndependentStatConfig>,
}

#[derive(Deserialize, Clone, Debug, Reflect)]
pub struct LevelUpConfig {
    pub value: f32,
    pub growth_factor: f32,
    pub growth_strategy: GrowthStrategy,
}

#[derive(Deserialize, Clone, Debug, Reflect, Default)]
pub struct LevelScaledStats {
    pub void_shards_reward: LevelScaledStat,
    pub spawn_timer: LevelScaledStat,
    pub enemy_health: LevelScaledStat,
    pub base_enemy_speed: LevelScaledStat,
    pub base_enemy_lifetime: LevelScaledStat,
}

#[derive(Deserialize, Clone, Debug, Reflect, Default)]
pub struct LevelScaledStat {
    pub value: f32,
    pub growth_factor: f32,
    pub growth_strategy: GrowthStrategy,
}

impl LevelScaledStat {
    pub fn calculate(&self, level: u32) -> f32 {
        // Levels start at 1, so level 1 should rely on base value.
        // Formula usually involves (level - 1) for scaling from base.
        let effective_level = if level > 0 { (level - 1) as f32 } else { 0.0 };

        self.growth_strategy
            .calculate(self.value, effective_level, self.growth_factor)
    }
}

#[derive(Deserialize, Clone, Debug, Reflect)]
pub struct IndependentStatConfig {
    pub value: f32,
    pub price: f32,
    pub growth_factor: f32,
    pub price_growth_factor: f32,
    pub growth_strategy: GrowthStrategy,
    pub price_growth_strategy: GrowthStrategy,
}
