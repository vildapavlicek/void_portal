use {bevy::prelude::*, game_core::VoidPortalPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Void Portal".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VoidPortalPlugin)
        .run();
}
