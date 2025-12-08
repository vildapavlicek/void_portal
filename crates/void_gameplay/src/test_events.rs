use bevy::prelude::*;
use crate::{
    configs::{EnemyConfig, GlobalConfig, SoldierConfig},
    portal::{despawn_dead_enemies, Enemy, Health, Reward},
};
use void_core::events::EnemyKilled;

#[test]
fn test_enemy_killed_event() {
    let mut app = App::new();

    // Add minimal plugins and core events
    app.add_plugins(MinimalPlugins);
    app.add_message::<EnemyKilled>(); // Bevy 0.17+ uses messages

    // Add the system we want to test
    app.add_systems(Update, despawn_dead_enemies);

    // Spawn an enemy that is already "dead" (0 health) and has a reward
    let reward_amount = 10.0;
    let enemy_entity = app
        .world_mut()
        .spawn((
            Enemy {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 0.0, // Dead
                max: 100.0,
            },
            Reward(reward_amount),
        ))
        .id();

    // Run the app one update cycle
    app.update();

    // Verify the enemy is despawned
    assert!(
        app.world().get_entity(enemy_entity).is_err(),
        "Enemy should be despawned"
    );

    // Verify the event was emitted
    // In Bevy 0.17+ we can use MessageCursor to read messages manually,
    // but here we use a helper system for simplicity.

    // We need to construct a cursor manually or use a system to read it.
    // Easier way in tests is often to run a system that reads events and asserts.
    // But let's try to access it via world resources directly if possible.

    // Since direct access seems tricky with just 'events.reader()', let's use a small helper system to capture events.
    #[derive(Resource, Default)]
    struct CapturedEvents(Vec<EnemyKilled>);

    app.init_resource::<CapturedEvents>();

    app.add_systems(Update, |mut events: MessageReader<EnemyKilled>, mut captured: ResMut<CapturedEvents>| {
        for event in events.read() {
            captured.0.push(event.clone());
        }
    });

    // Run update again to let the reader system pick up the events
    // Note: Messages sent in one frame are usually available in the next frame or same frame depending on ordering.
    // despawn_dead_enemies runs in Update. The reader also runs in Update.
    // If reader runs after despawn, it catches it.
    // Let's ensure order or just run again.
    app.update();

    let captured = app.world().resource::<CapturedEvents>();
    assert_eq!(captured.0.len(), 1, "Should emit exactly one EnemyKilled event");
    assert_eq!(captured.0[0].reward, reward_amount, "Reward amount should match");
}
