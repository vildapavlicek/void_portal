#![allow(clippy::type_complexity)]

use {
    bevy::prelude::*,
    common::GameState,
    items::{AttackRange as ItemAttackRange, BaseDamage, Item, Melee, Ranged},
    player_npcs::{MovementSpeed, PlayerNpc, Weapon, WeaponCooldown},
};

pub struct PlayerNpcsUiPlugin;

impl Plugin for PlayerNpcsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (attach_soldier_ui_observer, close_soldier_ui_actions)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
struct SoldierClickObserverAttached;

#[derive(Component)]
struct SoldierUiRoot;

#[derive(Component)]
struct SoldierUiCloseButton;

// Attach observer to PlayerNpc entities
fn attach_soldier_ui_observer(
    mut commands: Commands,
    query: Query<Entity, (With<PlayerNpc>, Without<SoldierClickObserverAttached>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(SoldierClickObserverAttached)
            .observe(on_soldier_click);
    }
}

// Event handler for Soldier click
fn on_soldier_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    soldier_query: Query<(&MovementSpeed, &Children)>,
    weapon_query: Query<
        (
            &Item,
            &BaseDamage,
            &ItemAttackRange,
            &WeaponCooldown,
            Option<&Melee>,
            Option<&Ranged>,
        ),
        With<Weapon>,
    >,
    ui_query: Query<Entity, With<SoldierUiRoot>>,
) {
    // If UI is already open, don't spawn another one
    if !ui_query.is_empty() {
        return;
    }

    let entity = trigger.entity;
    if let Ok((speed, children)) = soldier_query.get(entity) {
        // Find Weapon
        let mut weapon_info = None;
        for &child in children {
            if let Ok((item, damage, range, cooldown, melee, ranged)) = weapon_query.get(child) {
                let weapon_type = if melee.is_some() {
                    "Melee"
                } else if ranged.is_some() {
                    "Ranged"
                } else {
                    "Unknown"
                };
                weapon_info = Some((
                    item.name.clone(),
                    weapon_type,
                    damage.0,
                    range.0,
                    cooldown.timer.duration().as_secs_f32(),
                ));
                break; // Assuming one weapon for now
            }
        }

        spawn_soldier_ui(&mut commands, speed.0, weapon_info);
    }
}

fn spawn_soldier_ui(
    commands: &mut Commands,
    movement_speed: f32,
    weapon_info: Option<(String, &str, f32, f32, f32)>, // Name, Type, Damage, Range, Cooldown
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
            SoldierUiRoot,
        ))
        .observe(on_scrim_click) // Close on scrim click
        .with_children(|parent| {
            // The Panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Auto,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Start,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::hsla(210.0, 0.5, 0.2, 0.9)), // Dark Blue/Grey, semi-transparent
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .observe(block_click) // Block clicks from reaching the scrim
                .with_children(|p| {
                    // Title
                    p.spawn((
                        Text::new("Soldier Stats"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Basic Stats
                    // Inlined logic for spawn_stat_row("Movement Speed:", &format!("{:.1}", movement_speed));
                    p.spawn((Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },))
                        .with_children(|row| {
                            row.spawn((
                                Text::new("Movement Speed:"),
                                TextFont::default(),
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                            row.spawn((
                                Text::new(format!("{:.1}", movement_speed)),
                                TextFont::default(),
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Weapon Section
                    if let Some((name, w_type, damage, range, cooldown)) = weapon_info {
                        p.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(1.0),
                                margin: UiRect::axes(Val::Px(0.0), Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::WHITE),
                        ));

                        p.spawn((
                            Text::new("Equipped Weapon"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.5)),
                        ));

                        let stats = [
                            ("Name:", name.clone()),
                            ("Type:", w_type.to_string()),
                            ("Damage:", format!("{:.1}", damage)),
                            ("Range:", format!("{:.1}", range)),
                            ("Cooldown:", format!("{:.2}s", cooldown)),
                        ];

                        for (label, value) in stats {
                            p.spawn((Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },))
                                .with_children(|row| {
                                    row.spawn((
                                        Text::new(label),
                                        TextFont::default(),
                                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                    ));
                                    row.spawn((
                                        Text::new(value),
                                        TextFont::default(),
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        }
                    } else {
                        p.spawn((
                            Text::new("No Weapon Equipped"),
                            TextFont::default(),
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                    }

                    // Close Button
                    p.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::hsla(210.0, 0.6, 0.4, 1.0)),
                        BorderRadius::all(Val::Px(5.0)),
                        SoldierUiCloseButton,
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
    query: Query<Entity, With<SoldierUiRoot>>,
) {
    if let Ok(entity) = query.get(trigger.entity) {
        commands.entity(entity).despawn();
    }
}

// Block clicks from propagating
fn block_click(mut trigger: On<Pointer<Click>>) {
    trigger.propagate(false);
}

fn close_soldier_ui_actions(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &ChildOf),
        (Changed<Interaction>, With<SoldierUiCloseButton>),
    >,
    parent_query: Query<&ChildOf>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    root_query: Query<Entity, With<SoldierUiRoot>>,
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
    use {super::*, bevy::state::app::StatesPlugin};

    #[test]
    fn test_soldier_ui_spawn() {
        let mut app = App::new();
        // Minimal set of plugins for UI testing
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .add_plugins(AssetPlugin::default())
            // Events needed for UI
            .add_message::<bevy::input::mouse::MouseButtonInput>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_message::<bevy::window::CursorMoved>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            // Needed components
            .register_type::<PlayerNpc>()
            .register_type::<MovementSpeed>()
            .register_type::<Weapon>();

        app.init_state::<GameState>();
        app.add_plugins(PlayerNpcsUiPlugin);
        app.insert_state(GameState::Playing);

        // Spawn a dummy soldier
        let soldier = app
            .world_mut()
            .spawn((
                PlayerNpc,
                MovementSpeed(100.0),
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        app.update();

        // Check if observer attached
        assert!(app
            .world()
            .get::<SoldierClickObserverAttached>(soldier)
            .is_some());

        // Test spawning UI manually
        spawn_soldier_ui(
            &mut app.world_mut().commands(),
            100.0,
            Some(("Test Sword".to_string(), "Melee", 10.0, 30.0, 1.0)),
        );
        app.update();

        // Check if UI Root exists
        let mut ui_roots = app.world_mut().query::<&SoldierUiRoot>();
        assert_eq!(ui_roots.iter(app.world()).count(), 1);
    }
}
