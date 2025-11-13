pub mod assets;

use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

pub use assets::{AnimationBlendingConfig, AnimationBlendingConfigLoader};

pub(super) fn plugin(app: &mut App) {
    // Register the asset loader for RON config files
    app.init_asset::<AnimationBlendingConfig>();
    app.init_asset_loader::<AnimationBlendingConfigLoader>();

    // Load animation blending configuration
    app.load_resource::<AnimationBlendingConfig>();
}
