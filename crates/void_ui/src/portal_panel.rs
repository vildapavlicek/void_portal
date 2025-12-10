use {
    bevy::prelude::*,
    void_core::GameState,
    void_gameplay::portal::{Portal, UpgradeCoef, UpgradePrice, VoidShardsReward},
};

pub struct PortalPanelPlugin;

impl Plugin for PortalPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (attach_portal_observer, close_portal_ui_actions)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// Marker components
#[derive(Component)]
struct PortalClickObserverAttached;

#[derive(Component)]
struct PortalUiRoot;

#[derive(Component)]
struct PortalUiCloseButton;

#[derive(Component)]
struct PortalUiScrim;

// Attach observer to Portal entities
fn attach_portal_observer(
    mut commands: Commands,
    query: Query<Entity, (With<Portal>, Without<PortalClickObserverAttached>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(PortalClickObserverAttached)
            .observe(on_portal_click);
    }
}

// Event handler for Portal click
fn on_portal_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<(
        &VoidShardsReward,
        &UpgradePrice,
        &UpgradeCoef,
    )>,
    ui_query: Query<Entity, With<PortalUiRoot>>,
) {
    // If UI is already open, don't spawn another one
    if !ui_query.is_empty() {
        return;
    }

    let entity = trigger.entity;
    if let Ok((reward, price, coef)) = query.get(entity) {
        spawn_portal_ui(&mut commands, reward.0, price.0, coef.0);
    }
}

fn spawn_portal_ui(commands: &mut Commands, reward: f32, price: f32, coef: f32) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(100),
            // Invisible background to catch clicks outside the panel
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            PortalUiRoot,
            PortalUiScrim,
        ))
        .observe(on_scrim_click) // Close on scrim click
        .with_children(|parent| {
            // The Panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::hsla(270.0, 0.5, 0.2, 0.9)), // Dark purple, semi-transparent
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .observe(block_click) // Block clicks from reaching the scrim
                .with_children(|p| {
                    // Title
                    p.spawn((
                        Text::new("Portal Stats"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Stats
                    p.spawn((
                        Text::new(format!("Void Shards Reward: {:.2}", reward)),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                    p.spawn((
                        Text::new(format!("Upgrade Price: {:.2}", price)),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                    p.spawn((
                        Text::new(format!("Upgrade Coef: {:.2}", coef)),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));

                    // Close Button
                    p.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::hsla(270.0, 0.6, 0.4, 1.0)),
                        BorderRadius::all(Val::Px(5.0)),
                        PortalUiCloseButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Close"),
                            TextFont::default(),
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });
}

// Close when clicking the Scrim (root)
fn on_scrim_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<Entity, With<PortalUiRoot>>,
) {
    if let Ok(entity) = query.get(trigger.entity) {
        commands.entity(entity).despawn();
    }
}

// Block clicks from propagating
fn block_click(mut trigger: Trigger<Pointer<Click>>) {
    trigger.propagate(false);
}

fn close_portal_ui_actions(
    mut commands: Commands,
    // Close button interaction
    mut interaction_query: Query<
        (&Interaction, &ChildOf),
        (Changed<Interaction>, With<PortalUiCloseButton>),
    >,
    // Hierarchy helper
    parent_query: Query<&ChildOf>,
    // Keyboard input
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Root query for closing via keyboard
    root_query: Query<Entity, With<PortalUiRoot>>,
) {
    // Handle Close Button
    for (interaction, button_child_of) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // button_child_of.parent() is the Panel
            let panel_entity = button_child_of.parent();

            // We need to find the Root (parent of Panel)
            if let Ok(panel_child_of) = parent_query.get(panel_entity) {
                let root_entity = panel_child_of.parent();
                commands.entity(root_entity).despawn();
            }
            return;
        }
    }

    // Handle ESC
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in &root_query {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use void_gameplay::configs::PortalConfig;

    #[test]
    fn test_portal_ui_lifecycle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(StatesPlugin)
           .add_plugins(AssetPlugin::default());

        app.init_resource::<ButtonInput<KeyCode>>(); // Manually init input

        // Setup States and Resources
        app.init_state::<GameState>();
        app.insert_resource(PortalConfig {
            spawn_timer: 1.0,
            base_void_shards_reward: 10.0,
            base_upgrade_price: 100.0,
            upgrade_price_increase_coef: 1.5,
            portal_top_offset: 50.0,
            base_enemy_health: 10.0,
            base_enemy_speed: 10.0,
            base_enemy_lifetime: 10.0,
            base_enemy_reward: 1.0,
        });

        // Add Plugin
        app.add_plugins(PortalPanelPlugin);

        // Transition to Playing
        app.insert_state(GameState::Playing);

        // Mock a Portal
        let portal = app
            .world_mut()
            .spawn((
                Portal,
                VoidShardsReward(10.0),
                UpgradePrice(100.0),
                UpgradeCoef(1.5),
            ))
            .id();

        app.update();

        // Check if observer attached
        assert!(app.world().get::<PortalClickObserverAttached>(portal).is_some());

        // Simulate Click by spawning UI directly
        spawn_portal_ui(&mut app.world_mut().commands(), 10.0, 100.0, 1.5);
        app.update();

        // Check if UI Spawned
        let ui_root = app
            .world_mut()
            .query_filtered::<Entity, With<PortalUiRoot>>()
            .iter(app.world())
            .next();
        assert!(ui_root.is_some(), "UI Root should exist after manual spawn");

        // Simulate Esc
        // Manually update input before running update
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update(); // Process system

         // Check if UI Despawned
        let ui_root = app
            .world_mut()
            .query_filtered::<Entity, With<PortalUiRoot>>()
            .iter(app.world())
            .next();
        assert!(ui_root.is_none(), "UI Root should be gone after Esc");
    }
}
