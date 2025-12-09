use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Portal;

#[derive(Component, Reflect)]
pub struct VoidShardsReward(pub f32);

#[derive(Component, Reflect)]
pub struct UpgradePrice(pub f32);

#[derive(Component, Reflect)]
pub struct UpgradeCoef(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct EnemySpawner {
    pub timer: Timer,
}
