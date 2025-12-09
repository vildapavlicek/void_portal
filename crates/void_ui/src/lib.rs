use {
    bevy::prelude::*,
    void_core::{GameState, VoidCorePlugin},
    void_wallet::Wallet,
    void_components::ui::{WalletText, WalletUiRoot},
};

pub struct VoidUiPlugin;

impl Plugin for VoidUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
            app.add_plugins(VoidCorePlugin);
        }

        app.add_systems(OnEnter(GameState::Playing), spawn_wallet_ui)
            .add_systems(Update, update_wallet_ui.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_wallet_ui);

        info!("Void UI initialized");
    }
}

fn spawn_wallet_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::hsla(270.0, 0.5, 0.8, 0.5)),
            WalletUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("void shards: 0"),
                TextFont::default(),
                TextColor(Color::WHITE),
                WalletText,
            ));
        });
}

fn update_wallet_ui(wallet: Res<Wallet>, mut query: Query<&mut Text, With<WalletText>>) {
    if wallet.is_changed() {
        for mut text in &mut query {
            **text = format!("void shards: {}", wallet.void_shards);
        }
    }
}

fn despawn_wallet_ui(mut commands: Commands, query: Query<Entity, With<WalletUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn test_wallet_ui_lifecycle() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
           .add_plugins(StatesPlugin)
           .add_plugins(AssetPlugin::default());

        app.init_resource::<Wallet>();
        app.init_state::<GameState>();

        app.add_systems(OnEnter(GameState::Playing), spawn_wallet_ui)
            .add_systems(Update, update_wallet_ui.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_wallet_ui);

        // Start app
        app.update();

        // State is Loading
        assert_eq!(*app.world().resource::<State<GameState>>(), GameState::Loading);

        // UI should not exist
        assert!(app.world_mut().query::<&WalletText>().iter(app.world()).next().is_none());

        // Transition to Playing
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update(); // Transition
        app.update(); // OnEnter

        // Verify UI spawned
        let mut query = app.world_mut().query_filtered::<Entity, With<WalletText>>();
        let text_entity = query.single(app.world()).unwrap();
        let text = app.world().get::<Text>(text_entity).unwrap();
        assert_eq!(**text, "void shards: 0");

        // Update wallet
        app.world_mut().resource_mut::<Wallet>().void_shards = 100.5;
        app.update();

        let text = app.world().get::<Text>(text_entity).unwrap();
        assert_eq!(**text, "void shards: 100.5");

        // Transition back to Loading
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Loading);
        app.update(); // Transition
        app.update(); // OnExit

        // Verify UI despawned
        assert!(app.world_mut().query::<&WalletText>().iter(app.world()).next().is_none());
    }
}
