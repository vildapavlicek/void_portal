use bevy::prelude::*;
use void_assets::VoidAssetsPlugin;
use void_core::VoidCorePlugin;
use void_gameplay::VoidGameplayPlugin;
use void_ui::VoidUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Void Portal".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VoidCorePlugin)
        .add_plugins(VoidAssetsPlugin)
        .add_plugins(VoidUiPlugin)
        .add_plugins(VoidGameplayPlugin)
        .run();
}
