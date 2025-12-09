use {bevy::prelude::*, bevy::asset::LoadedFolder};

#[derive(Resource, Default)]
pub struct PrefabHandles {
    pub portal: Handle<Scene>,
    pub enemies_folder: Handle<LoadedFolder>,
    pub soldier: Handle<Scene>,
}
