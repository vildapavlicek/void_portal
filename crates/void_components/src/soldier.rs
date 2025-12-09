use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Soldier {
    pub attack_timer: Timer,
    pub target: Option<Entity>,
}

#[derive(Component, Reflect)]
pub struct AttackRange(pub f32);

#[derive(Component)]
pub struct Moving(pub Entity);

#[derive(Component)]
pub struct Attacking(pub Entity);

#[derive(Component, Reflect)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: Timer,
}
