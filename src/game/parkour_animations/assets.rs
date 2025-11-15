use bevy::{gltf::Gltf, prelude::*};

/// Resource holding parkour animation GLTF files
#[derive(Resource, Asset, Reflect, Clone)]
pub struct ParkourGltfAssets {
    /// The vault animation GLTF
    #[dependency]
    pub vault_gltf: Handle<Gltf>,
}

impl ParkourGltfAssets {
    /// Path to the vault animation GLB file
    pub const VAULT_PATH: &'static str = "models/animations/vault.glb";
}

impl FromWorld for ParkourGltfAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            vault_gltf: assets.load(Self::VAULT_PATH),
        }
    }
}

/// Resource holding extracted parkour animation clips
/// Created once the GLTFs are fully loaded
#[derive(Resource, Clone)]
pub struct ParkourAnimations {
    /// Animation clips extracted from the GLTFs
    pub vault: Handle<AnimationClip>,
    pub climb: Handle<AnimationClip>,
    pub slide: Handle<AnimationClip>,
    pub wall_run_left: Handle<AnimationClip>,
    pub wall_run_right: Handle<AnimationClip>,
    pub roll: Handle<AnimationClip>,
}

/// Extracts animation clips from loaded parkour GLTFs
/// This system runs once the ParkourGltfAssets are loaded
pub fn extract_parkour_animation_clips(
    mut commands: Commands,
    gltf_assets_handle: Res<ParkourGltfAssets>,
    gltf_assets: Res<Assets<Gltf>>,
    parkour_animations: Option<Res<ParkourAnimations>>,
    asset_server: Res<AssetServer>,
) {
    // Only run once - if ParkourAnimations already exists, we're done
    if parkour_animations.is_some() {
        return;
    }

    // Try to get the loaded vault GLTF
    let Some(vault_gltf) = gltf_assets.get(&gltf_assets_handle.vault_gltf) else {
        // GLTF not loaded yet, try again next frame
        return;
    };

    // Extract vault animation (should be the first animation in the file)
    let Some(vault) = vault_gltf.animations.first().cloned() else {
        error!("Vault GLTF does not contain any animations!");
        return;
    };

    info!("✅ Extracted vault animation from GLTF");
    info!("   Animation count in vault.glb: {}", vault_gltf.animations.len());

    // For now, load other animations using the old method
    // TODO: Convert all to GLTF loader pattern
    let climb = asset_server.load(
        bevy::gltf::GltfAssetLabel::Animation(0).from_asset("models/animations/Freehang Climb.glb")
    );
    let slide = asset_server.load(
        bevy::gltf::GltfAssetLabel::Animation(0).from_asset("models/animations/Running Slide.glb")
    );
    let wall_run_left = asset_server.load(
        bevy::gltf::GltfAssetLabel::Animation(0).from_asset("models/animations/Over Obstacle Jumping.glb")
    );
    let wall_run_right = asset_server.load(
        bevy::gltf::GltfAssetLabel::Animation(0).from_asset("models/animations/Over Obstacle Jumping.glb")
    );
    let roll = asset_server.load(
        bevy::gltf::GltfAssetLabel::Animation(0).from_asset("models/animations/Falling To Roll.glb")
    );

    // Create ParkourAnimations resource with extracted clips
    commands.insert_resource(ParkourAnimations {
        vault,
        climb,
        slide,
        wall_run_left,
        wall_run_right,
        roll,
    });

    info!("🎉 Parkour animations extracted and ready!");
}
