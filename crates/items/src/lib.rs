use bevy::prelude::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .register_type::<Armor>()
            .register_type::<Melee>()
            .register_type::<Ranged>()
            .register_type::<BaseDamage>()
            .register_type::<AttackRange>()
            .register_type::<AttackSpeed>()
            .register_type::<ProjectileStats>()
            .register_type::<AreaOfEffect>();
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Item {
    pub name: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Armor {
    pub defense: f32,
    pub movement_speed_modifier: f32,
}

// New Components

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Melee;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Ranged;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct BaseDamage(pub f32);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct AttackRange(pub f32);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct AttackSpeed(pub f32);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct ProjectileStats {
    pub speed: f32,
    pub lifetime: f32,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct AreaOfEffect {
    pub radius: f32,
    pub angle: f32,
}
