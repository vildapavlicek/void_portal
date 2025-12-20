use {
    crate::{ConditionalUpgrade, GrowthStrategy},
    bevy::prelude::*,
    std::collections::HashMap,
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PortalRoot; // Marker for querying

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct PortalLevel {
    pub active: u32,
    pub max_unlocked: u32,
}

#[derive(Component, Reflect, Default, Debug)]
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

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct BaseMonsterHealth(pub GrowthStrategy);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct BaseMonsterReward(pub GrowthStrategy);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct BaseMonsterSpeed(pub GrowthStrategy);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct BaseMonsterLifetime(pub GrowthStrategy);

#[derive(Component, Reflect, Default, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct BaseMonsterArmor(pub GrowthStrategy);

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

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct LockedFeature;
