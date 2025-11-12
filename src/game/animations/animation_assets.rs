use bevy::prelude::*;

/// Resource holding GLTF handles (will extract animations once loaded)
#[derive(Resource, Asset, Reflect, Clone)]
pub struct PlayerAnimationGltfs {
    #[dependency]
    pub idle_gltf: Handle<Gltf>,
    #[dependency]
    pub run_gltf: Handle<Gltf>,
    #[dependency]
    pub jump_gltf: Handle<Gltf>,
    #[dependency]
    pub falling_gltf: Handle<Gltf>,
}

impl FromWorld for PlayerAnimationGltfs {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            idle_gltf: asset_server.load("models/animations/Breathing Idle.glb"),
            run_gltf: asset_server.load("models/animations/Standard Run.glb"),
            jump_gltf: asset_server.load("models/animations/Standing Jumping.glb"),
            falling_gltf: asset_server.load("models/animations/Hard Landing.glb"),
        }
    }
}

/// Resource holding animation clip handles (extracted after GLTF loads)
#[derive(Resource, Clone)]
pub struct PlayerAnimations {
    pub idle: Handle<AnimationClip>,
    pub run: Handle<AnimationClip>,
    pub jump: Handle<AnimationClip>,
    pub falling: Handle<AnimationClip>,
}
