use bevy::{gltf::Gltf, prelude::*};

/// Resource holding the main player GLTF (contains both model and animations)
#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerGltfAsset {
    /// The main player GLTF file containing model and animations
    #[dependency]
    pub gltf: Handle<Gltf>,
}

impl PlayerGltfAsset {
    /// Path to the player character GLB file
    pub const PATH: &'static str = "models/characters/brian_parkour.glb";
}

impl FromWorld for PlayerGltfAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            gltf: assets.load(Self::PATH),
        }
    }
}

/// Resource holding extracted player assets (scene and animations)
/// Created once the GLTF is fully loaded
#[derive(Resource, Clone)]
pub struct PlayerAssets {
    /// The player character scene
    pub character_scene: Handle<Scene>,
    /// Animation clips extracted from the GLTF
    pub animations: PlayerAnimations,
}

/// Animation clips extracted from the player GLTF
#[derive(Clone)]
pub struct PlayerAnimations {
    pub idle: Handle<AnimationClip>,
    pub running: Handle<AnimationClip>,
    pub standing_jump: Handle<AnimationClip>,
    pub running_jump: Handle<AnimationClip>,
    pub walking: Handle<AnimationClip>,

}

/// Extracts scene and animations from the loaded player GLTF
/// This system runs once the PlayerGltfAsset is loaded
pub fn extract_player_assets(
    mut commands: Commands,
    gltf_asset: Res<PlayerGltfAsset>,
    gltf_assets: Res<Assets<Gltf>>,
    player_assets: Option<Res<PlayerAssets>>,
) {
    // Only run once - if PlayerAssets already exists, we're done
    if player_assets.is_some() {
        return;
    }

    // Try to get the loaded GLTF
    let Some(gltf) = gltf_assets.get(&gltf_asset.gltf) else {
        // GLTF not loaded yet, try again next frame
        return;
    };

    // Extract the character scene (first scene in the GLTF)
    let Some(character_scene) = gltf.scenes.first().cloned() else {
        error!("Player GLTF does not contain any scenes!");
        return;
    };

    info!("Found {} animations in player GLTF", gltf.animations.len());

    // Debug: Print animation names to help with mapping
    for (idx, anim) in gltf.named_animations.iter().enumerate() {
        info!("Animation {}: {}", idx, anim.0);
    }

    // Extract animations by name from the GLTF
    // Mixamo animation names: idle, running, standing_jump, running_jump
    let idle = gltf.named_animations.get("idle")
        .or_else(|| gltf.named_animations.get("Idle"))
        .cloned();
    let running = gltf.named_animations.get("running")
        .or_else(|| gltf.named_animations.get("Running"))
        .cloned();
    let standing_jump = gltf.named_animations.get("standing_jump")
        .or_else(|| gltf.named_animations.get("Standing Jump"))
        .cloned();
    let running_jump = gltf.named_animations.get("running_jump")
        .or_else(|| gltf.named_animations.get("Running Jump"))
        .cloned();
    let walking = gltf.named_animations.get("walk").cloned();
    // Verify we got all required animations
    let (Some(idle), Some(running), Some(standing_jump), Some(running_jump), Some(walking)) =
        (idle, running, standing_jump, running_jump, walking)
    else {
        error!("Failed to extract all required animations from player GLTF!");
        error!("Expected animation names: idle, running, standing_jump, running_jump");
        error!("Available animations: {:?}", gltf.named_animations.keys().collect::<Vec<_>>());
        return;
    };

    // Create PlayerAssets resource with extracted data
    let assets = PlayerAssets {
        character_scene,
        animations: PlayerAnimations {
            idle,
            running,
            standing_jump,
            running_jump,
            walking,
        },
    };

    commands.insert_resource(assets);
    info!("Successfully extracted player scene and animations from unified GLTF!");
}
