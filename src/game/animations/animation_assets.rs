use bevy::prelude::*;

/// Resource holding handles to all animation clips from Mixamo
#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAnimations {
    // Basic locomotion
    #[dependency]
    pub idle: Handle<AnimationClip>,
    #[dependency]
    pub run: Handle<AnimationClip>,
    #[dependency]
    pub jump: Handle<AnimationClip>,
    #[dependency]
    pub falling: Handle<AnimationClip>,

    // Parkour animations (for future phases)
    #[dependency]
    pub jump_to_hang: Handle<AnimationClip>,
    #[dependency]
    pub freehang_climb: Handle<AnimationClip>,
    #[dependency]
    pub vault: Handle<AnimationClip>,
    #[dependency]
    pub slide: Handle<AnimationClip>,
    #[dependency]
    pub landing_roll: Handle<AnimationClip>,
}

impl FromWorld for PlayerAnimations {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            // Basic locomotion clips
            idle: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Breathing Idle.glb")),
            run: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Standard Run.glb")),
            jump: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Standing Jumping.glb")),
            falling: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Hard Landing.glb")),

            // Parkour clips (loaded but not used yet)
            jump_to_hang: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Jump To Hang.glb")),
            freehang_climb: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Freehang Climb.glb")),
            vault: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Over Obstacle Jumping.glb")),
            slide: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Running Slide.glb")),
            landing_roll: asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/animations/Falling To Roll.glb")),
        }
    }
}
