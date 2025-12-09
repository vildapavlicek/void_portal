use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Reflect)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

#[derive(Component, Reflect)]
pub struct Damage(pub f32); // Generalized damage if needed, though Projectile has its own field currently.
