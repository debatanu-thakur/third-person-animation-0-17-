mod assets;
use crate::{
    asset_tracking::LoadResource,
    game::{animations::models::AnimationState, foot_placement::FootPlacementEnabled, target_matching::{BoneMap, TargetMatchEnabled}, third_person_camera::ThirdPersonCameraTarget},
    screens::Screen,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub use assets::{PlayerAnimations, PlayerAssets, PlayerGltfAsset};
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian3d::*;

// Player marker component
#[derive(Component)]
pub struct Player;

// Movement state
#[derive(Component)]
pub struct MovementController {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub sprint_multiplier: f32,
    pub jump_velocity: f32,
    pub jump_height: f32,
    pub double_jump_available: bool,
    pub is_grounded: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            walk_speed: 2.0,  // Walking speed (default movement)
            run_speed: 8.0,   // Running speed (when Shift is held)
            sprint_multiplier: 1.5,
            jump_velocity: 22.0, // Increased from 8.0 for more responsive jumping
            jump_height: 4.0, // Increased from 8.0 for more responsive jumping
            double_jump_available: false,
            is_grounded: false,
        }
    }
}

// Constants
pub const PLAYER_HEIGHT: f32 = 0.94;
pub const PLAYER_RADIUS: f32 = 0.34;

// Player spawn command
pub struct SpawnPlayer {
    pub position: Vec3,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let _ = world.run_system_cached_with(spawn_player, self);
    }
}

fn spawn_player(
    In(spawn_config): In<SpawnPlayer>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
) {
    commands
        .spawn((
            Name::new("Player"),
            Player,
            MovementController::default(),
            ThirdPersonCameraTarget, // Tells camera to follow this entity
            DespawnOnExit(Screen::Gameplay), // Cleanup when leaving Gameplay screen
            Transform::from_translation(spawn_config.position),
            Visibility::Visible,
            FootPlacementEnabled::for_testing(),
            TargetMatchEnabled,              // Enables target matching
            BoneMap::default(),              // Auto-populates with foot bones
        ))
        .insert((
            // Avian3D physics components
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, PLAYER_HEIGHT),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_RADIUS*0.99, 0.0)),
            TnuaAnimatingState::<AnimationState>::default(),
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        ))
        .with_children(|parent| {
            parent.spawn((
                 // Prevent player from tipping over
                SceneRoot(player_assets.character_scene.clone()),
                Transform::from_translation(Vec3::new(0., -0.9, 0.))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
            ));
        });
}

pub(super) fn plugin(app: &mut App) {
    // Load player GLTF (contains model + animations)
    app.load_resource::<PlayerGltfAsset>();

    // Extract assets from loaded GLTF
    app.add_systems(
        Update,
        assets::extract_player_assets
            .run_if(resource_added::<PlayerGltfAsset>)
    );

    // Set stronger gravity for faster falling (default is -9.81)
    app.insert_resource(Gravity(Vec3::new(0.0, -100.0, 0.0)));
}
