use {
    assets::VoidAssetsPlugin,
    bevy::{asset::LoadedFolder, prelude::*},
    common::GameState,
    enemy::{AvailableEnemies, EnemyConfig, EnemyPlugin},
    portal::{PortalConfig, PortalPlugin},
    soldier::{SoldierConfig, SoldierPlugin},
    ui::VoidUiPlugin,
    wallet::VoidWalletPlugin,
};

pub struct VoidPortalPlugin;

#[derive(Resource, Default)]
struct GameConfigHandles {
    portal: Handle<PortalConfig>,
    enemies_folder: Handle<LoadedFolder>,
    soldier: Handle<SoldierConfig>,
}

impl Plugin for VoidPortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();

        app.add_plugins((
            VoidAssetsPlugin,
            VoidWalletPlugin,
            EnemyPlugin,
            PortalPlugin,
            SoldierPlugin,
            VoidUiPlugin,
        ));

        app.init_resource::<GameConfigHandles>();

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
    handles.portal = asset_server.load("configs/main.portal.ron");
    handles.enemies_folder = asset_server.load_folder("configs/enemies");
    handles.soldier = asset_server.load("configs/main.soldier.ron");

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
    portal_assets: Res<Assets<PortalConfig>>,
    soldier_assets: Res<Assets<SoldierConfig>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    enemy_assets: Res<Assets<EnemyConfig>>,
    mut available_enemies: ResMut<AvailableEnemies>,
    mut next_state: ResMut<NextState<GameState>>,
    loading_text_query: Query<Entity, With<LoadingText>>,
) {
    if let (Some(portal), Some(soldier), Some(enemies_folder)) = (
        portal_assets.get(&handles.portal),
        soldier_assets.get(&handles.soldier),
        loaded_folders.get(&handles.enemies_folder),
    ) {
        commands.insert_resource(portal.clone());
        commands.insert_resource(soldier.clone());

        available_enemies.0.clear();
        for handle in &enemies_folder.handles {
            let typed_handle: Handle<EnemyConfig> = handle.clone().typed();
            if let Some(config) = enemy_assets.get(&typed_handle) {
                available_enemies.0.push(config.clone());
            }
        }

        if available_enemies.0.is_empty() {
            warn!("No enemies loaded from configs/enemies/");
        } else {
            info!("Loaded {} enemy configs", available_enemies.0.len());
        }

        for entity in loading_text_query.iter() {
            commands.entity(entity).despawn();
        }

        info!("Configs loaded. Transitioning to Playing.");
        next_state.set(GameState::Playing);
    }
}

#[cfg(test)]
mod tests_integration;
