use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct EnemyKilled {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct UpgradePortal;

#[derive(Message, Debug, Clone)]
pub struct RequestUpgrade {
    pub upgrade_entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct ChangeActiveLevel {
    pub portal_entity: Entity,
    pub change: i32, // +1 or -1
}

#[derive(Message, Debug, Clone)]
pub struct EnemyScavenged {
    pub amount: f32,
}
