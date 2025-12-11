use {
    bevy::prelude::*,
    common::{GameState, UpgradePortal},
    portal::{Portal, UpgradeCoef, UpgradePrice, VoidShardsReward},
    wallet::Wallet,
};

pub struct PortalPanelPlugin;

impl Plugin for PortalPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                attach_portal_observer,
                close_portal_ui_actions,
                update_upgrade_button_state,
                update_portal_ui_stats,
            )
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
struct PortalUiUpgradeButton;

#[derive(Component)]
struct PortalUiScrim;

#[derive(Component)]
struct PortalUiLink(Entity);

#[derive(Component, Clone, Copy)]
enum PortalUiStat {
    Reward,
    Price,
    Coef,
}

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
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    query: Query<(&VoidShardsReward, &UpgradePrice, &UpgradeCoef)>,
    ui_query: Query<Entity, With<PortalUiRoot>>,
    wallet: Res<Wallet>,
) {
    // If UI is already open, don't spawn another one
    if !ui_query.is_empty() {
        return;
    }

    let entity = trigger.entity;
    if let Ok((reward, price, coef)) = query.get(entity) {
        spawn_portal_ui(
            &mut commands,
            reward.0,
            price.0,
            coef.0,
            entity,
            wallet.void_shards,
        );
    }
}

fn spawn_portal_ui(
    commands: &mut Commands,
    reward: f32,
    price: f32,
    coef: f32,
    portal_entity: Entity,
    current_funds: f32,
) {
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
                        height: Val::Px(240.0), // Increased height for new button
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
                        PortalUiStat::Reward,
                        PortalUiLink(portal_entity),
                    ));
                    p.spawn((
                        Text::new(format!("Upgrade Price: {:.2}", price)),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                        PortalUiStat::Price,
                        PortalUiLink(portal_entity),
                    ));
                    p.spawn((
                        Text::new(format!("Upgrade Coef: {:.2}", coef)),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                        PortalUiStat::Coef,
                        PortalUiLink(portal_entity),
                    ));

                    // Upgrade Button
                    p.spawn((
                        Button,
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        // Initial color based on funds
                        BackgroundColor(if current_funds >= price {
                            Color::hsla(120.0, 0.6, 0.4, 1.0) // Green
                        } else {
                            Color::hsla(0.0, 0.0, 0.5, 1.0) // Grey
                        }),
                        BorderRadius::all(Val::Px(5.0)),
                        PortalUiUpgradeButton,
                        PortalUiLink(portal_entity),
                    ))
                    .observe(on_upgrade_click)
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("Upgrade ({:.0})", price)),
                            TextFont::default(),
                            TextColor(Color::WHITE),
                        ));
                    });

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
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    query: Query<Entity, With<PortalUiRoot>>,
) {
    if let Ok(entity) = query.get(trigger.entity) {
        commands.entity(entity).despawn();
    }
}

// Block clicks from propagating
fn block_click(mut trigger: On<Pointer<Click>>) {
    trigger.propagate(false);
}

// Handle Upgrade Click
fn on_upgrade_click(
    trigger: On<Pointer<Click>>,
    mut messages: MessageWriter<UpgradePortal>,
    button_query: Query<&PortalUiLink>,
    portal_query: Query<&UpgradePrice, With<Portal>>,
    wallet: Res<Wallet>,
) {
    let button_entity = trigger.entity;

    if let Ok(link) = button_query.get(button_entity) {
        if let Ok(upgrade_price) = portal_query.get(link.0) {
            if wallet.void_shards >= upgrade_price.0 {
                messages.write(UpgradePortal);
            }
        }
    }
}

// Update Upgrade Button State (Color and Text)
fn update_upgrade_button_state(
    mut button_query: Query<
        (&PortalUiLink, &mut BackgroundColor, &Children),
        With<PortalUiUpgradeButton>,
    >,
    mut text_query: Query<&mut Text>,
    portal_query: Query<&UpgradePrice, With<Portal>>,
    wallet: Res<Wallet>,
) {
    for (link, mut bg_color, children) in &mut button_query {
        if let Ok(upgrade_price) = portal_query.get(link.0) {
            let price = upgrade_price.0;
            let affordable = wallet.void_shards >= price;

            // Update Color
            *bg_color = if affordable {
                BackgroundColor(Color::hsla(120.0, 0.6, 0.4, 1.0)) // Green
            } else {
                BackgroundColor(Color::hsla(0.0, 0.0, 0.5, 1.0)) // Grey
            };

            // Update Text
            for &child in children {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("Upgrade ({:.0})", price);
                }
            }
        }
    }
}

// Update Stat Texts
fn update_portal_ui_stats(
    mut query: Query<(&PortalUiLink, &PortalUiStat, &mut Text)>,
    portal_query: Query<(&VoidShardsReward, &UpgradePrice, &UpgradeCoef), With<Portal>>,
) {
    for (link, stat_type, mut text) in &mut query {
        if let Ok((reward, price, coef)) = portal_query.get(link.0) {
            match stat_type {
                PortalUiStat::Reward => **text = format!("Void Shards Reward: {:.2}", reward.0),
                PortalUiStat::Price => **text = format!("Upgrade Price: {:.2}", price.0),
                PortalUiStat::Coef => **text = format!("Upgrade Coef: {:.2}", coef.0),
            }
        }
    }
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
    use {super::*, bevy::state::app::StatesPlugin, portal::PortalConfig};

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
            enemy_health_growth_factor: 0.5,
            enemy_reward_growth_factor: 1.0,
        });

        app.insert_resource(Wallet {
            void_shards: 1000.0,
        });
        app.add_message::<UpgradePortal>();

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
        assert!(app
            .world()
            .get::<PortalClickObserverAttached>(portal)
            .is_some());

        // Simulate Click by spawning UI directly
        // Pass wallet funds manually or query from resource in test context
        spawn_portal_ui(
            &mut app.world_mut().commands(),
            10.0,
            100.0,
            1.5,
            portal,
            1000.0,
        );
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
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
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
