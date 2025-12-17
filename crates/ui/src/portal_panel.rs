use {
    bevy::prelude::*,
    common::{
        components::{MonsterScaling, PortalLevel, UpgradeCost, UpgradeSlot},
        ChangeActiveLevel, GameState, RequestUpgrade, UpgradePortal, UpgradeableStat,
    },
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
    Generic,
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
    Generic,
}

#[derive(Component)]
struct PortalLevelControl {
    direction: i32, // -1 or +1
}

// Attach observer to Portal entities
fn attach_portal_observer(
    mut commands: Commands,
    query: Query<Entity, (With<PortalLevel>, Without<PortalClickObserverAttached>)>,
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
    portal_query: Query<(&PortalLevel, &UpgradeCost, &MonsterScaling, &Children)>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
    ui_query: Query<Entity, With<PortalUiRoot>>,
) {
    // If UI is already open, don't spawn another one
    if !ui_query.is_empty() {
        return;
    }

    let entity = trigger.entity;
    if let Ok((level, cost, scaling, children)) = portal_query.get(entity) {
        let mut upgrades = Vec::new();
        for &child in children {
            if let Ok((slot, stat)) = upgrade_query.get(child) {
                upgrades.push((child, slot.clone(), stat.clone()));
            }
        }

        spawn_portal_ui(&mut commands, level, cost, scaling, entity, upgrades);
    }
}

fn spawn_portal_ui(
    commands: &mut Commands,
    level: &PortalLevel,
    cost: &UpgradeCost,
    scaling: &MonsterScaling,
    portal_entity: Entity,
    upgrades: Vec<(Entity, UpgradeSlot, UpgradeableStat)>,
) {
    let current_reward = scaling.reward_strategy.calculate(level.active as f32);

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

                    // --- Section 1: Portal Level Control ---
                    p.spawn((Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },))
                        .with_children(|row| {
                            // Left Side: Active / Max + Controls
                            row.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|control_row| {
                                // Decrease Button
                                control_row
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(30.0),
                                            height: Val::Px(30.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::hsla(0.0, 0.0, 0.5, 1.0)),
                                        BorderRadius::all(Val::Px(5.0)),
                                        PortalLevelControl { direction: -1 },
                                        PortalUiLink(portal_entity),
                                    ))
                                    .observe(on_level_control_click)
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("<"),
                                            TextFont::default(),
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                // Level Text
                                control_row
                                    .spawn((Node {
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },))
                                    .with_children(|col| {
                                        col.spawn((
                                            Text::new(format!(
                                                "Level {} / {}",
                                                level.active, level.max_unlocked
                                            )),
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

                                // Increase Button
                                control_row
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(30.0),
                                            height: Val::Px(30.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::hsla(0.0, 0.0, 0.5, 1.0)),
                                        BorderRadius::all(Val::Px(5.0)),
                                        PortalLevelControl { direction: 1 },
                                        PortalUiLink(portal_entity),
                                    ))
                                    .observe(on_level_control_click)
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(">"),
                                            TextFont::default(),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });

                            // Right Side: Upgrade Button
                            spawn_upgrade_button(
                                row,
                                PortalUpgradeTarget::Level,
                                cost.current_price,
                                portal_entity,
                            );
                        });

                    // --- Dynamic Generic Sections ---
                    for (child_entity, slot, stat) in upgrades {
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
                                    Text::new(format!("{}: {:.2}", slot.name, stat.value)),
                                    TextFont::default(),
                                    TextColor(Color::WHITE),
                                    PortalUiStat::Generic,
                                    PortalUiLink(child_entity),
                                ));

                                spawn_upgrade_button(
                                    row,
                                    PortalUpgradeTarget::Generic,
                                    stat.price,
                                    child_entity,
                                );
                            });
                    }

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

fn on_level_control_click(
    trigger: On<Pointer<Click>>,
    mut events: MessageWriter<ChangeActiveLevel>,
    query: Query<(&PortalLevelControl, &PortalUiLink)>,
) {
    if let Ok((control, link)) = query.get(trigger.entity) {
        events.write(ChangeActiveLevel {
            portal_entity: link.0,
            change: control.direction,
        });
    }
}

// Handle Upgrade Click
fn on_upgrade_click(
    trigger: On<Pointer<Click>>,
    mut msg_level: MessageWriter<UpgradePortal>,
    mut msg_generic: MessageWriter<RequestUpgrade>,
    button_query: Query<(&PortalUiLink, &PortalUiUpgradeButton)>,
    portal_query: Query<(&PortalLevel, &UpgradeCost)>,
    stat_query: Query<&UpgradeableStat>,
    wallet: Res<Wallet>,
) {
    let button_entity = trigger.entity;

    if let Ok((link, button_type)) = button_query.get(button_entity) {
        match button_type.0 {
            PortalUpgradeTarget::Level => {
                if let Ok((_, cost)) = portal_query.get(link.0) {
                    if wallet.void_shards >= cost.current_price {
                        msg_level.write(UpgradePortal);
                    }
                }
            }
            PortalUpgradeTarget::Generic => {
                if let Ok(stat) = stat_query.get(link.0) {
                    if wallet.void_shards >= stat.price {
                        msg_generic.write(RequestUpgrade {
                            upgrade_entity: link.0,
                        });
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
    portal_query: Query<&UpgradeCost>,
    stat_query: Query<&UpgradeableStat>,
    wallet: Res<Wallet>,
) {
    for (link, button_type, mut bg_color, children) in &mut button_query {
        let price_opt = match button_type.0 {
            PortalUpgradeTarget::Level => portal_query.get(link.0).map(|c| c.current_price).ok(),
            PortalUpgradeTarget::Generic => stat_query.get(link.0).map(|s| s.price).ok(),
        };

        if let Some(price) = price_opt {
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
    portal_query: Query<(&PortalLevel, &MonsterScaling)>,
    upgrade_query: Query<(&UpgradeSlot, &UpgradeableStat)>,
) {
    for (link, stat_type, mut text) in &mut query {
        match stat_type {
            PortalUiStat::Level => {
                if let Ok((level, _)) = portal_query.get(link.0) {
                    **text = format!("Level {} / {}", level.active, level.max_unlocked);
                }
            }
            PortalUiStat::Reward => {
                if let Ok((level, scaling)) = portal_query.get(link.0) {
                    let current_reward = scaling.reward_strategy.calculate(level.active as f32);
                    **text = format!("Reward: {:.2}", current_reward);
                }
            }
            PortalUiStat::Generic => {
                if let Ok((slot, stat)) = upgrade_query.get(link.0) {
                    **text = format!("{}: {:.2}", slot.name, stat.value);
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
    use {super::*, bevy::state::app::StatesPlugin, common::GrowthStrategy};

    #[test]
    fn test_portal_ui_lifecycle() {
        use portal::{MonsterScaling, PortalLevel, UpgradeCost};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .add_plugins(AssetPlugin::default());

        app.init_resource::<ButtonInput<KeyCode>>(); // Manually init input

        // Setup States and Resources
        app.init_state::<GameState>();

        app.insert_resource(Wallet {
            void_shards: 1000.0,
        });
        app.add_message::<UpgradePortal>();
        app.add_message::<RequestUpgrade>();
        app.add_message::<ChangeActiveLevel>();

        // Add Plugin
        app.add_plugins(PortalPanelPlugin);

        // Transition to Playing
        app.insert_state(GameState::Playing);

        // Mock a Portal using new components
        let portal_ent = app
            .world_mut()
            .spawn((
                PortalLevel {
                    active: 1,
                    max_unlocked: 1,
                },
                UpgradeCost {
                    strategy: GrowthStrategy::Linear {
                        base: 100.0,
                        coefficient: 1.5,
                    },
                    current_price: 100.0,
                },
                MonsterScaling::default(), // Can populate if needed
            ))
            .id();

        app.update();

        // Check if observer attached
        assert!(app
            .world()
            .get::<PortalClickObserverAttached>(portal_ent)
            .is_some());

        // Test spawning UI manually
        app.world_mut().spawn((Node::default(), PortalUiRoot));

        app.update();

        // Check if UI Root exists
        let ui_root = app
            .world_mut()
            .query_filtered::<Entity, With<PortalUiRoot>>()
            .iter(app.world())
            .next();
        assert!(ui_root.is_some());

        // Simulate Esc
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
