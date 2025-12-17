use {
    bevy::prelude::*,
    common::{GameState, SpawnFloatingText, VoidGameStage},
};

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnFloatingText>();

        app.register_type::<FloatingText>()
            .register_type::<FloatingTextAnim>();

        app.add_systems(
            Update,
            (
                spawn_floating_text.in_set(VoidGameStage::Effect),
                animate_floating_text.in_set(VoidGameStage::Actions),
                cleanup_floating_text.in_set(VoidGameStage::FrameEnd),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Components
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct FloatingText;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct FloatingTextAnim {
    pub lifetime: Timer,
    pub velocity: Vec3,
}

// Systems
fn spawn_floating_text(mut commands: Commands, mut messages: MessageReader<SpawnFloatingText>) {
    for msg in messages.read() {
        // random x/y jitter could be nice, but not strictly requested.
        // I'll add a tiny random z-offset to prevent z-fighting if multiple texts spawn at exact same spot.
        // But I don't have rand here easily without rand crate.
        // I'll just use a high Z index.

        commands.spawn((
            FloatingText,
            FloatingTextAnim {
                lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                velocity: Vec3::new(0.0, 40.0, 0.0), // Move up
            },
            Text2d::new(msg.text.clone()),
            TextFont {
                font_size: msg.size,
                ..default()
            },
            TextColor(msg.color),
            // Ensure Z is high enough to be seen over sprites (often 0-100).
            // UI is usually camera dependent, but Text2d is in world.
            Transform::from_translation(msg.location + Vec3::new(0.0, 0.0, 200.0)),
        ));
    }
}

fn animate_floating_text(
    _commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut TextColor, &FloatingTextAnim), With<FloatingText>>,
) {
    for (mut transform, mut color, anim) in query.iter_mut() {
        transform.translation += anim.velocity * time.delta_secs();

        // Fade out based on lifetime.
        // We need to read the timer from anim, but timer is ticked in cleanup?
        // Or we tick it here?
        // If we tick it here, cleanup needs to read it.
        // Better to tick in one place.
        // I'll tick in cleanup. Here I assume timer is current.
        // But cleanup runs in FrameEnd, this runs in Actions.
        // So timer is from previous frame or hasn't ticked yet for this frame.
        // That's fine.

        let percent_left = anim.lifetime.fraction_remaining();
        color.0.set_alpha(percent_left);
    }
}

fn cleanup_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FloatingTextAnim), With<FloatingText>>,
) {
    for (entity, mut anim) in query.iter_mut() {
        anim.lifetime.tick(time.delta());
        if anim.lifetime.is_finished() {
            // Bevy 0.17 EntityCommands::despawn is recursive by default or despawn_recursive is replaced?
            // The memory says: "EntityCommands::despawn() is recursive and removes the entity along with all its children automatically."
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests;
