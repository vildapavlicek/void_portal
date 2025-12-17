use {
    crate::*,
    bevy::state::app::StatesPlugin,
    common::{SpawnFloatingText, VoidGameStage},
    std::{thread, time::Duration},
};

#[test]
fn test_spawn_floating_text() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin); // Required for init_state
    app.add_plugins(VfxPlugin);

    // We need to initialize state and schedule
    app.init_state::<GameState>();

    app.configure_sets(
        Update,
        (
            VoidGameStage::ResolveIntent,
            VoidGameStage::Actions,
            VoidGameStage::Effect,
            VoidGameStage::FrameEnd,
        )
            .chain()
            .run_if(in_state(GameState::Playing)),
    );

    // Enter Playing state
    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::Playing);
    app.update(); // Apply state change
    app.update(); // Run schedule once in playing

    // Send Message
    let mut events = app
        .world_mut()
        .resource_mut::<Messages<SpawnFloatingText>>();
    events.write(SpawnFloatingText::create("Test", Vec3::ZERO, Color::WHITE));

    // Run app to process message (spawn text in Effect stage)
    app.update();

    // Verify entity spawned
    let mut query = app.world_mut().query::<(&Text2d, &FloatingText)>();
    let (text, _) = query.single(app.world()).unwrap();
    assert_eq!(text.0, "Test");
}

#[test]
fn test_animate_and_cleanup() {
    let mut app = App::new();
    // Use standard MinimalPlugins which includes TimePlugin
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(VfxPlugin);

    app.init_state::<GameState>();
    app.configure_sets(
        Update,
        (
            VoidGameStage::ResolveIntent,
            VoidGameStage::Actions,
            VoidGameStage::Effect,
            VoidGameStage::FrameEnd,
        )
            .chain()
            .run_if(in_state(GameState::Playing)),
    );

    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::Playing);
    app.update();
    app.update();

    // Spawn manually to test animation
    let entity = app
        .world_mut()
        .spawn((
            FloatingText,
            FloatingTextAnim {
                lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                velocity: Vec3::new(0.0, 10.0, 0.0),
            },
            Text2d::new("Anim"),
            TextFont::default(),
            TextColor(Color::WHITE),
            Transform::default(),
        ))
        .id();

    // First update to initialize time (delta might be zero or small)
    app.update();

    // Sleep to simulate time passing
    thread::sleep(Duration::from_millis(100));
    app.update();

    // Check position moved
    let transform = app.world().get::<Transform>(entity).unwrap();
    println!("Position y: {}", transform.translation.y);
    // Should be > 0. 100ms * 10 = 1.0 unit approx.
    assert!(transform.translation.y > 0.0);

    // Check alpha faded (should be < 1.0)
    let color = app.world().get::<TextColor>(entity).unwrap();
    assert!(color.0.alpha() < 1.0);

    // To test cleanup, we can't easily wait 1s in test without slowing it down.
    // We can manually tick the timer in a loop or force the component value.
    // Let's force the timer to be finished.
    {
        let mut anim = app.world_mut().get_mut::<FloatingTextAnim>(entity).unwrap();
        anim.lifetime.set_elapsed(Duration::from_secs(2));
    }
    app.update();

    // Check despawned
    assert!(app.world().get_entity(entity).is_err());
}
