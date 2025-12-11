#![allow(clippy::type_complexity)]

use bevy::prelude::*;

pub struct VoidAssetsPlugin;

impl Plugin for VoidAssetsPlugin {
    fn build(&self, _app: &mut App) {
        info!("Void Assets initialized");
    }
}
