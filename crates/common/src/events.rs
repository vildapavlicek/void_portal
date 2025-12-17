use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct SpawnEnemyRequest {
    pub portal_entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct MonsterKilled {
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
pub struct MonsterScavenged {
    pub amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum DamageType {
    Physical,
    Magic,
    Void,
}

#[derive(Message, Debug, Clone)]
pub struct DamageMessage {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}
