use bevy::prelude::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .register_type::<Weapon>()
            .register_type::<Armor>();
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Item {
    pub name: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Weapon {
    pub damage: f32,
    pub range: f32,
    pub attack_speed_modifier: f32,
    pub projectile_speed: f32,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Armor {
    pub defense: f32,
    pub movement_speed_modifier: f32,
}

/// Helper to spawn a weapon entity.
pub fn spawn_weapon(
    commands: &mut Commands,
    name: &str,
    damage: f32,
    range: f32,
    attack_speed_modifier: f32,
    projectile_speed: f32,
) -> Entity {
    commands
        .spawn((
            Item {
                name: name.to_string(),
            },
            Weapon {
                damage,
                range,
                attack_speed_modifier,
                projectile_speed,
            },
        ))
        .id()
}
