use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext};
use serde::Deserialize;
use std::marker::PhantomData;

#[derive(Asset, TypePath, Deserialize, Debug)]
pub struct EnemyConfig {
    pub spawn_timer: f32,
    pub lifetime: f32,
    pub hp: f32,
    pub speed: f32,
}

#[derive(Asset, TypePath, Deserialize, Debug)]
pub struct SoldierConfig {
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub projectile_speed: f32,
}

pub struct RonLoader<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for RonLoader<T> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> AssetLoader for RonLoader<T>
where
    T: Asset + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    type Asset = T;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

pub struct VoidConfigPlugin;

impl Plugin for VoidConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<EnemyConfig>()
           .init_asset::<SoldierConfig>()
           .register_asset_loader(RonLoader::<EnemyConfig>::default())
           .register_asset_loader(RonLoader::<SoldierConfig>::default());
    }
}
