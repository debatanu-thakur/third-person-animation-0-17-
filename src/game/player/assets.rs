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

    // Debug animation slots for bone extraction (optional)
    // Map these to numeric keys 1-0 for testing parkour animations
    pub debug_slot_1: Option<Handle<AnimationClip>>, // e.g., standing_vault
    pub debug_slot_2: Option<Handle<AnimationClip>>, // e.g., running_vault
    pub debug_slot_3: Option<Handle<AnimationClip>>, // e.g., climb_up
    pub debug_slot_4: Option<Handle<AnimationClip>>, // e.g., wall_run_left
    pub debug_slot_5: Option<Handle<AnimationClip>>, // e.g., wall_run_right
    pub debug_slot_6: Option<Handle<AnimationClip>>, // e.g., slide
    pub debug_slot_7: Option<Handle<AnimationClip>>, // e.g., roll
    pub debug_slot_8: Option<Handle<AnimationClip>>, // Reserved
    pub debug_slot_9: Option<Handle<AnimationClip>>, // Reserved
    pub debug_slot_0: Option<Handle<AnimationClip>>, // Reserved
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

    // Extract optional debug animation slots for bone extraction
    // These can be added to the GLTF temporarily for pose extraction
    let debug_slot_1 = gltf.named_animations.get("debug_1")
        .or_else(|| gltf.named_animations.get("standing_vault")).cloned();
    let debug_slot_2 = gltf.named_animations.get("debug_2")
        .or_else(|| gltf.named_animations.get("running_vault")).cloned();
    let debug_slot_3 = gltf.named_animations.get("debug_3")
        .or_else(|| gltf.named_animations.get("climb_up")).cloned();
    let debug_slot_4 = gltf.named_animations.get("debug_4")
        .or_else(|| gltf.named_animations.get("wall_run_left")).cloned();
    let debug_slot_5 = gltf.named_animations.get("debug_5")
        .or_else(|| gltf.named_animations.get("wall_run_right")).cloned();
    let debug_slot_6 = gltf.named_animations.get("debug_6")
        .or_else(|| gltf.named_animations.get("slide")).cloned();
    let debug_slot_7 = gltf.named_animations.get("debug_7")
        .or_else(|| gltf.named_animations.get("roll")).cloned();
    let debug_slot_8 = gltf.named_animations.get("debug_8").cloned();
    let debug_slot_9 = gltf.named_animations.get("debug_9").cloned();
    let debug_slot_0 = gltf.named_animations.get("debug_0").cloned();

    if debug_slot_1.is_some() || debug_slot_2.is_some() || debug_slot_3.is_some() {
        info!("DEBUG: Found parkour debug animations for bone extraction!");
        if debug_slot_1.is_some() { info!("  - Slot 1 (press '1')"); }
        if debug_slot_2.is_some() { info!("  - Slot 2 (press '2')"); }
        if debug_slot_3.is_some() { info!("  - Slot 3 (press '3')"); }
        if debug_slot_4.is_some() { info!("  - Slot 4 (press '4')"); }
        if debug_slot_5.is_some() { info!("  - Slot 5 (press '5')"); }
        if debug_slot_6.is_some() { info!("  - Slot 6 (press '6')"); }
        if debug_slot_7.is_some() { info!("  - Slot 7 (press '7')"); }
        info!("Press F12 to dump current bone transforms to RON file");
    }

    // Create PlayerAssets resource with extracted data
    let assets = PlayerAssets {
        character_scene,
        animations: PlayerAnimations {
            idle,
            running,
            standing_jump,
            running_jump,
            walking,
            debug_slot_1,
            debug_slot_2,
            debug_slot_3,
            debug_slot_4,
            debug_slot_5,
            debug_slot_6,
            debug_slot_7,
            debug_slot_8,
            debug_slot_9,
            debug_slot_0,
        },
    };

    commands.insert_resource(assets);
    info!("Successfully extracted player scene and animations from unified GLTF!");
}
