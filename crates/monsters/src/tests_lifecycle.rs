use {
    crate::{manage_monster_lifecycle, process_marked_cleanup, Health, Lifetime, Monster},
    bevy::{prelude::*, time::TimePlugin},
    common::{MarkedForCleanUp, MonsterKilled, MonsterScavenged, Reward},
};

#[test]
fn test_monster_death_lifecycle() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());
    app.add_message::<MonsterKilled>();
    app.add_message::<MonsterScavenged>();

    // Helper to capture events
    #[derive(Resource, Default)]
    struct CapturedEvents(Vec<MonsterKilled>);

    app.init_resource::<CapturedEvents>();

    app.add_systems(
        Update,
        |mut events: MessageReader<MonsterKilled>, mut captured: ResMut<CapturedEvents>| {
            for event in events.read() {
                captured.0.push(event.clone());
            }
        },
    );

    // Add death systems
    // manage_monster_lifecycle runs, emits event, modifies entity.
    // event reader runs (order undefined unless explicit, but next frame definitely catches it).
    // despawn_dead_bodies runs.
    app.add_systems(Update, (manage_monster_lifecycle, process_marked_cleanup));

    // Spawn an monster with 0 health
    let reward_amount = 10.0;
    let monster_entity = app
        .world_mut()
        .spawn((
            Monster {
                target_position: Vec2::ZERO,
            },
            Health {
                current: 0.0,
                max: 100.0,
            },
            Lifetime::default(),
            Transform::default(),
            Reward(reward_amount),
            // Ensure Visibility is present to test Hidden
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Run 1 frame
    app.update();

    // 1. Verify Event emitted
    // We might need another update if reader runs before handler.
    // Or we can check if it's in the resource.
    // If undefined order, it might take 2 ticks for reader to see it.
    app.update();

    let captured = app.world().resource::<CapturedEvents>();
    assert_eq!(
        captured.0.len(),
        1,
        "Should emit exactly one MonsterKilled event"
    );
    assert_eq!(captured.0[0].entity, monster_entity);

    // 2. Verify Entity still exists
    assert!(app.world().get_entity(monster_entity).is_ok());

    // 3. Verify Monster component removed
    assert!(
        app.world().get::<Monster>(monster_entity).is_none(),
        "Monster component should be removed"
    );

    // 4. Verify MarkedForCleanUp component added
    assert!(
        app.world().get::<MarkedForCleanUp>(monster_entity).is_some(),
        "MarkedForCleanUp component should be added"
    );

    // 5. Verify Hidden
    let visibility = app.world().get::<Visibility>(monster_entity).unwrap();
    assert_eq!(
        *visibility,
        Visibility::Hidden,
        "Visibility should be Hidden"
    );

    // 6. Advance time 0.9s (Timer is 1.0s)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.9));
    }
    app.update();

    // Verify still exists
    assert!(
        app.world().get_entity(monster_entity).is_ok(),
        "Entity should still exist after 0.9s"
    );

    // 7. Advance time 0.2s (Total 1.1s)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.2));
    }
    app.update();

    // Verify Despawned
    assert!(
        app.world().get_entity(monster_entity).is_err(),
        "Entity should be despawned after 1.1s"
    );
}
