use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct SpawnMonsterRequest {
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
    pub location: Vec3,
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

#[derive(Message, Debug, Clone)]
pub struct MeleeHitMessage {
    pub attacker: Entity,
    pub target: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct ProjectileCollisionMessage {
    pub projectile: Entity,
    pub source: Entity,
    pub target: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct SpawnFloatingText {
    pub text: String,
    pub location: Vec3,
    pub color: Color,
    pub size: f32,
}

impl SpawnFloatingText {
    pub fn create(text: impl Into<String>, location: Vec3, color: Color) -> Self {
        Self {
            text: text.into(),
            location,
            color,
            size: 20.0,
        }
    }

    pub fn damage(amount: f32, location: Vec3) -> Self {
        let text = format!("{:.0}", amount);

        Self {
            text,
            location,
            color: bevy::color::palettes::basic::RED.into(),
            size: 24.0,
        }
    }

    pub fn void_shards_reward(amount: f32, location: Vec3) -> Self {
        Self {
            text: format!("+{:.0}", amount),
            location,
            color: Color::srgb(0.5, 0.0, 0.5),
            size: 20.0,
        }
    }
}
