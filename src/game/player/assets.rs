use bevy::prelude::*;

/// Resource holding handles to player character assets
#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAssets {
    /// The player character 3D model (GLB file)
    #[dependency]
    pub character_model: Handle<Scene>,
}

#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAnimationGltf {
    #[dependency]
    pub player_gltf: Handle<Gltf>,
}

impl PlayerAssets {
    /// Path to the player character GLB file
    pub const PATH_CHARACTER: &'static str = "models/characters/brian.glb#Scene0";
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            character_model: assets
                .load(GltfAssetLabel::Scene(0).from_asset(PlayerAssets::PATH_CHARACTER)),
        }
    }
}

impl FromWorld for PlayerAnimationGltf {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player_gltf: assets.load(""),
        }
    }
}

#[derive(Resource, Reflect, Clone)]
pub struct ObjectGltf {
    pub gltf: Handle<Gltf>,
}
