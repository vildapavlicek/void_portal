use {crate::GrowthStrategy, bevy::prelude::*, std::collections::HashMap};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PortalRoot; // Marker for querying

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PortalLevel {
    pub active: u32,
    pub max_unlocked: u32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct UpgradeCost {
    pub strategy: GrowthStrategy,
    pub current_price: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PortalSpawner {
    pub timer: Timer,
    pub interval_strategy: GrowthStrategy,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MonsterScaling {
    pub health_strategy: GrowthStrategy,
    pub reward_strategy: GrowthStrategy,
    pub speed_strategy: GrowthStrategy,
    pub lifetime_strategy: GrowthStrategy,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ScavengerPenalty(pub f32);

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct UpgradeSlot {
    pub name: String,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct PortalUpgrades(pub HashMap<String, Entity>);
