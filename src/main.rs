use {
    bevy::prelude::*, void_assets::VoidAssetsPlugin, void_core::VoidCorePlugin,
    void_gameplay::VoidGameplayPlugin, void_ui::VoidUiPlugin, void_wallet::VoidWalletPlugin,
};

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
