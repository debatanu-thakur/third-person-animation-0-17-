use bevy::{
    asset::{AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Animation blending configuration loaded from RON file
#[derive(Asset, Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct AnimationBlendingConfig {
    /// Speed thresholds for animation transitions
    pub speed_thresholds: SpeedThresholds,
}

/// Speed thresholds that control animation blending
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct SpeedThresholds {
    /// Below this speed, character is considered idle
    pub idle_threshold: f32,
    /// Speed at which walk animation is at 100%
    pub walk_speed: f32,
    /// Speed at which run animation is at 100%
    pub run_speed: f32,
}

impl AnimationBlendingConfig {
    /// Path to the animation blending configuration file
    pub const PATH: &'static str = "config/animation_blending.ron";
}

impl Default for AnimationBlendingConfig {
    fn default() -> Self {
        Self {
            speed_thresholds: SpeedThresholds {
                idle_threshold: 0.1,
                walk_speed: 2.0,
                run_speed: 8.0,
            },
        }
    }
}

/// Asset loader for AnimationBlendingConfig RON files
#[derive(Default)]
pub struct AnimationBlendingConfigLoader;

impl AssetLoader for AnimationBlendingConfigLoader {
    type Asset = AnimationBlendingConfig;
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
        let config: AnimationBlendingConfig = ron::de::from_bytes(&bytes)?;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
