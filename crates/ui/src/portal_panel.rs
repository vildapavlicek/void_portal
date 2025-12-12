use {
    bevy::prelude::*,
    common::{GameState, UpgradePortal, UpgradePortalBonusLifetime, UpgradePortalCapacity},
    portal::{Portal, PortalBonusLifetime, PortalCapacity, PortalConfig},
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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum PortalUpgradeTarget {
    Level,
    Capacity,
    Lifetime,
}

#[derive(Component)]
struct PortalUiUpgradeButton(PortalUpgradeTarget);

#[derive(Component)]
struct PortalUiScrim;

#[derive(Component)]
struct PortalUiLink(Entity);

#[derive(Component, Clone, Copy)]
enum PortalUiStat {
    Level,
    Reward,
    Capacity,
    Lifetime,
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
    query: Query<(&Portal, &PortalCapacity, &PortalBonusLifetime)>,
    ui_query: Query<Entity, With<PortalUiRoot>>,
    portal_config: Res<PortalConfig>,
) {
    // If UI is already open, don't spawn another one
    if !ui_query.is_empty() {
        return;
    }

    let entity = trigger.entity;
    if let Ok((portal, capacity, lifetime)) = query.get(entity) {
        spawn_portal_ui(
            &mut commands,
            portal,
            capacity,
            lifetime,
            entity,
            &portal_config,
        );
    }
}

fn spawn_portal_ui(
    commands: &mut Commands,
    portal: &Portal,
    capacity: &PortalCapacity,
    lifetime: &PortalBonusLifetime,
    portal_entity: Entity,
    config: &PortalConfig,
) {
    let current_reward = config
        .level_scaled_stats
        .void_shards_reward
        .calculate(portal.level);

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
                        width: Val::Px(400.0),
                        height: Val::Px(350.0),
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

                    // --- Section 1: Portal Level ---
                    p.spawn((Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },))
                        .with_children(|row| {
                            row.spawn((Node {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },))
                                .with_children(|col| {
                                    col.spawn((
                                        Text::new(format!("Level {}", portal.level)),
                                        TextFont::default(),
                                        TextColor(Color::WHITE),
                                        PortalUiStat::Level,
                                        PortalUiLink(portal_entity),
                                    ));
                                    col.spawn((
                                        Text::new(format!("Reward: {:.2}", current_reward)),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                        PortalUiStat::Reward,
                                        PortalUiLink(portal_entity),
                                    ));
                                });

                            spawn_upgrade_button(
                                row,
                                PortalUpgradeTarget::Level,
                                portal.upgrade_price,
                                portal_entity,
                            );
                        });

                    // --- Section 2: Capacity ---
                    p.spawn((Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!("Max Enemies: {:.0}", capacity.0.value)),
                                TextFont::default(),
                                TextColor(Color::WHITE),
                                PortalUiStat::Capacity,
                                PortalUiLink(portal_entity),
                            ));

                            spawn_upgrade_button(
                                row,
                                PortalUpgradeTarget::Capacity,
                                capacity.0.price,
                                portal_entity,
                            );
                        });

                    // --- Section 3: Bonus Lifetime ---
                    p.spawn((Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!("Bonus Lifetime: {:.2}s", lifetime.0.value)),
                                TextFont::default(),
                                TextColor(Color::WHITE),
                                PortalUiStat::Lifetime,
                                PortalUiLink(portal_entity),
                            ));

                            spawn_upgrade_button(
                                row,
                                PortalUpgradeTarget::Lifetime,
                                lifetime.0.price,
                                portal_entity,
                            );
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

fn spawn_upgrade_button(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    target: PortalUpgradeTarget,
    price: f32,
    link: Entity,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(140.0),
                height: Val::Px(35.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::hsla(0.0, 0.0, 0.5, 1.0)), // Initial Grey
            BorderRadius::all(Val::Px(5.0)),
            PortalUiUpgradeButton(target),
            PortalUiLink(link),
        ))
        .observe(on_upgrade_click)
        .with_children(|btn| {
            btn.spawn((
                Text::new(format!("Upgrade ({:.0})", price)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
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
    mut msg_level: MessageWriter<UpgradePortal>,
    mut msg_capacity: MessageWriter<UpgradePortalCapacity>,
    mut msg_lifetime: MessageWriter<UpgradePortalBonusLifetime>,
    button_query: Query<(&PortalUiLink, &PortalUiUpgradeButton)>,
    portal_query: Query<(&Portal, &PortalCapacity, &PortalBonusLifetime)>,
    wallet: Res<Wallet>,
) {
    let button_entity = trigger.entity;

    if let Ok((link, button_type)) = button_query.get(button_entity) {
        if let Ok((portal, capacity, lifetime)) = portal_query.get(link.0) {
            match button_type.0 {
                PortalUpgradeTarget::Level => {
                    if wallet.void_shards >= portal.upgrade_price {
                        msg_level.write(UpgradePortal);
                    }
                }
                PortalUpgradeTarget::Capacity => {
                    if wallet.void_shards >= capacity.0.price {
                        msg_capacity.write(UpgradePortalCapacity);
                    }
                }
                PortalUpgradeTarget::Lifetime => {
                    if wallet.void_shards >= lifetime.0.price {
                        msg_lifetime.write(UpgradePortalBonusLifetime);
                    }
                }
            }
        }
    }
}

// Update Upgrade Button State (Color and Text)
fn update_upgrade_button_state(
    mut button_query: Query<(
        &PortalUiLink,
        &PortalUiUpgradeButton,
        &mut BackgroundColor,
        &Children,
    )>,
    mut text_query: Query<&mut Text>,
    portal_query: Query<(&Portal, &PortalCapacity, &PortalBonusLifetime)>,
    wallet: Res<Wallet>,
) {
    for (link, button_type, mut bg_color, children) in &mut button_query {
        if let Ok((portal, capacity, lifetime)) = portal_query.get(link.0) {
            let price = match button_type.0 {
                PortalUpgradeTarget::Level => portal.upgrade_price,
                PortalUpgradeTarget::Capacity => capacity.0.price,
                PortalUpgradeTarget::Lifetime => lifetime.0.price,
            };

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
                    // Check if it's the specific upgrade logic
                    **text = format!("Upgrade ({:.0})", price);
                }
            }
        }
    }
}

// Update Stat Texts
fn update_portal_ui_stats(
    mut query: Query<(&PortalUiLink, &PortalUiStat, &mut Text)>,
    portal_query: Query<(&Portal, &PortalCapacity, &PortalBonusLifetime)>,
    portal_config: Res<PortalConfig>,
) {
    for (link, stat_type, mut text) in &mut query {
        if let Ok((portal, capacity, lifetime)) = portal_query.get(link.0) {
            match stat_type {
                PortalUiStat::Level => **text = format!("Level {}", portal.level),
                PortalUiStat::Reward => {
                    let current_reward = portal_config
                        .level_scaled_stats
                        .void_shards_reward
                        .calculate(portal.level);
                    **text = format!("Reward: {:.2}", current_reward);
                }
                PortalUiStat::Capacity => **text = format!("Max Enemies: {:.0}", capacity.0.value),
                PortalUiStat::Lifetime => {
                    **text = format!("Bonus Lifetime: {:.2}s", lifetime.0.value)
                }
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

        // Create a dummy config
        let config = PortalConfig {
            level: 1,
            level_up_price: portal::LevelUpConfig {
                value: 100.0,
                growth_factor: 1.5,
                growth_strategy: common::GrowthStrategy::Linear,
            },
            portal_top_offset: 50.0,
            level_scaled_stats: portal::LevelScaledStats {
                void_shards_reward: portal::LevelScaledStat {
                    value: 10.0,
                    growth_factor: 1.0,
                    growth_strategy: common::GrowthStrategy::Linear,
                },
                spawn_timer: portal::LevelScaledStat {
                    value: 1.0,
                    growth_factor: 0.1,
                    growth_strategy: common::GrowthStrategy::Linear,
                },
                enemy_health: portal::LevelScaledStat {
                    value: 10.0,
                    growth_factor: 1.0,
                    growth_strategy: common::GrowthStrategy::Linear,
                },
                base_enemy_speed: portal::LevelScaledStat {
                    value: 10.0,
                    growth_factor: 1.0,
                    growth_strategy: common::GrowthStrategy::Linear,
                },
                base_enemy_lifetime: portal::LevelScaledStat {
                    value: 10.0,
                    growth_factor: 1.0,
                    growth_strategy: common::GrowthStrategy::Linear,
                },
            },
            independently_leveled_stats: portal::IndependentlyLeveledStats {
                capacity: portal::IndependentStatConfig {
                    value: 5.0,
                    price: 50.0,
                    growth_factor: 1.0,
                    price_growth_factor: 1.5,
                    growth_strategy: common::GrowthStrategy::Linear,
                    price_growth_strategy: common::GrowthStrategy::Linear,
                },
                lifetime: portal::IndependentStatConfig {
                    value: 0.0,
                    price: 50.0,
                    growth_factor: 1.0,
                    price_growth_factor: 1.5,
                    growth_strategy: common::GrowthStrategy::Linear,
                    price_growth_strategy: common::GrowthStrategy::Linear,
                },
            },
        };

        app.insert_resource(config.clone());

        app.insert_resource(Wallet {
            void_shards: 1000.0,
        });
        app.add_message::<UpgradePortal>();
        app.add_message::<UpgradePortalCapacity>();
        app.add_message::<UpgradePortalBonusLifetime>();

        // Add Plugin
        app.add_plugins(PortalPanelPlugin);

        // Transition to Playing
        app.insert_state(GameState::Playing);

        // Mock a Portal
        let capacity = common::UpgradeableStat {
            value: 5.0,
            price: 50.0,
            ..default()
        };
        let lifetime = common::UpgradeableStat {
            value: 0.0,
            price: 50.0,
            ..default()
        };

        let portal_ent = app
            .world_mut()
            .spawn((
                Portal {
                    level: 1,
                    upgrade_price: 100.0,
                    ..default()
                },
                PortalCapacity(capacity),
                PortalBonusLifetime(lifetime),
            ))
            .id();

        app.update();

        // Check if observer attached
        assert!(app
            .world()
            .get::<PortalClickObserverAttached>(portal_ent)
            .is_some());

        // FIXME: mutable and immutable access at the same time
        let mut commands = app.world_mut().commands();
        let portal = app.world().get::<Portal>(portal_ent).unwrap();
        let capacity = app.world().get::<PortalCapacity>(portal_ent).unwrap();
        let lifetime = app.world().get::<PortalBonusLifetime>(portal_ent).unwrap();
        // Simulate Click by spawning UI directly
        // Pass wallet funds manually or query from resource in test context
        spawn_portal_ui(
            &mut commands,
            portal,
            capacity,
            lifetime,
            portal_ent,
            &config,
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
