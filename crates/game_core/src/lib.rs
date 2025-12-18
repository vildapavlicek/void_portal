#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use {
    assets::VoidAssetsPlugin,
    bevy::{asset::LoadedFolder, prelude::*},
    common::{
        CommonPlugin, GameState, MonsterKilled, RequestUpgrade, UpgradePortal, VoidGameStage,
    },
    items::ItemsPlugin,
    monster_factory::MonsterFactoryPlugin,
    monsters::{AvailableEnemies, MonsterConfig, MonsterPlugin},
    player_npcs::PlayerNpcsPlugin,
    player_npcs_ui::PlayerNpcsUiPlugin,
    portal::PortalPlugin,
    ui::VoidUiPlugin,
    vfx::VfxPlugin,
    wallet::VoidWalletPlugin,
};

pub struct VoidPortalPlugin;

#[derive(Resource, Default)]
struct GameConfigHandles {
    portal_scene: Handle<DynamicScene>,
    monsters_folder: Handle<LoadedFolder>,
}

impl Plugin for VoidPortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_message::<MonsterKilled>();
        app.add_message::<UpgradePortal>();
        app.add_message::<RequestUpgrade>();

        app.add_plugins((
            CommonPlugin,
            ItemsPlugin,
            MonsterFactoryPlugin,
            MonsterPlugin,
            PlayerNpcsPlugin,
            PlayerNpcsUiPlugin,
            PortalPlugin,
            VfxPlugin,
            VoidAssetsPlugin,
            VoidUiPlugin,
            VoidWalletPlugin,
        ));

        app.init_resource::<GameConfigHandles>();

        app.configure_sets(
            Update,
            (
                VoidGameStage::FrameStart,
                VoidGameStage::ResolveIntent,
                VoidGameStage::Actions,
                VoidGameStage::Effect,
                VoidGameStage::FrameEnd,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(Startup, (setup_camera, start_loading));
        app.add_systems(
            Update,
            check_assets_ready.run_if(in_state(GameState::Loading)),
        );

        info!("Void Portal Core initialized");
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    debug!("Camera setup complete");
}

fn start_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut handles: ResMut<GameConfigHandles>,
) {
    handles.portal_scene = asset_server.load("prefabs/portal.scn.ron");
    handles.monsters_folder = asset_server.load_folder("configs/monsters");

    commands.spawn((
        Text2d::new("Loading..."),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        LoadingText,
    ));
}

#[derive(Component)]
struct LoadingText;

fn check_assets_ready(
    mut commands: Commands,
    handles: Res<GameConfigHandles>,
    // We don't need to check for PortalConfig asset anymore, just that the scene handle is "ready" (which load returns immediately,
    // but to follow the pattern we might want to check load state.
    // However, for DynamicScene, usually we just spawn it.
    // We DO need to wait for monsters folder to be loaded to populate AvailableEnemies.
    loaded_folders: Res<Assets<LoadedFolder>>,
    monster_config_asset: Res<Assets<MonsterConfig>>,
    mut available_monsters: ResMut<AvailableEnemies>,
    mut next_state: ResMut<NextState<GameState>>,
    loading_text_query: Query<Entity, With<LoadingText>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    // Check if monsters are loaded
    if let Some(monsters_folder) = loaded_folders.get(&handles.monsters_folder) {
        // Also check if portal scene is loaded?
        // Not strictly required to access its content here (since we just spawn it),
        // but good for ensuring smooth transition.

        if !asset_server.is_loaded_with_dependencies(&handles.portal_scene) {
            return;
        }

        available_monsters.0.clear();
        for handle in &monsters_folder.handles {
            let typed_handle: Handle<MonsterConfig> = handle.clone().typed();
            if let Some(config) = monster_config_asset.get(&typed_handle) {
                available_monsters.0.push(config.clone());
            }
        }

        if available_monsters.0.is_empty() {
            warn!("No monsters loaded from configs/monsters/");
        } else {
            info!("Loaded {} monster configs", available_monsters.0.len());
        }

        // Spawn the Portal Scene
        scene_spawner.spawn_dynamic(handles.portal_scene.clone());

        for entity in loading_text_query.iter() {
            commands.entity(entity).despawn();
        }

        info!("Configs loaded. Transitioning to Playing.");
        next_state.set(GameState::Playing);
    }
}

#[cfg(test)]
mod tests_integration;
