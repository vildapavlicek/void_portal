use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct EnemyKilled {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct UpgradePortal;

#[derive(Message, Debug, Clone)]
pub struct UpgradePortalCapacity;

#[derive(Message, Debug, Clone)]
pub struct UpgradePortalBonusLifetime;
