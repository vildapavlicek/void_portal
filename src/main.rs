#![allow(clippy::type_complexity)]

use {
    bevy::{log::LogPlugin, prelude::*},
    game_core::VoidPortalPlugin,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Void Portal".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "error,player_npcs=trace,monsters=trace".into(),
                    level: bevy::log::Level::TRACE,
                    ..Default::default()
                }),
        )
        .add_plugins(VoidPortalPlugin)
        .run();
}
