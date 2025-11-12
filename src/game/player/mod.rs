mod assets;
mod movement;
use crate::{
    asset_tracking::LoadResource,
    game::{animations::models::AnimationState, third_person_camera::ThirdPersonCameraTarget},
    screens::Screen,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub use assets::{PlayerAnimationGltf, PlayerAssets};
use bevy_hotpatching_experiments::hot;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

// Player marker component
#[derive(Component)]
pub struct Player;

// Movement state
#[derive(Component)]
pub struct MovementController {
    pub speed: f32,
    pub sprint_multiplier: f32,
    pub jump_velocity: f32,
    pub jump_height: f32,
    pub double_jump_available: bool,
    pub is_grounded: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            speed: 1.0, // Increased from 5.0 for tighter controls
            sprint_multiplier: 1.5,
            jump_velocity: 22.0, // Increased from 8.0 for more responsive jumping
            jump_height: 2.0, // Increased from 8.0 for more responsive jumping
            double_jump_available: false,
            is_grounded: false,
        }
    }
}

// Constants
pub const PLAYER_HEIGHT: f32 = 1.1;
pub const PLAYER_RADIUS: f32 = 0.5;

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
    let scale: f32 = 0.01;
    commands
        .spawn((
            Name::new("Player"),
            Player,
            MovementController::default(),
            // animation::AnimationState::default(), // Start with Idle animation state
            ThirdPersonCameraTarget, // Tells camera to follow this entity
            DespawnOnExit(Screen::Gameplay), // Cleanup when leaving Gameplay screen
            Transform::from_translation(spawn_config.position),
            Visibility::Visible,
            // Avian3D physics components
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_HEIGHT / 2., PLAYER_RADIUS),
            TnuaController::default(),
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(), // Prevent player from tipping over
            TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_HEIGHT / 2., 0.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(player_assets.character_model.clone()),
                Transform::from_translation(Vec3::new(0., -0.8, 0.))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                    .with_scale(Vec3::splat(scale)), // <-- Scale only visuals
            ));
        });
}

pub(super) fn plugin(app: &mut App) {
    // Load player assets
    app.load_resource::<PlayerAssets>();
    // Add Avian3D physics plugin with custom gravity
    // app.add_systems(Update, movement::player_movement);
    // Set stronger gravity for faster falling (default is -9.81)
    app.insert_resource(Gravity(Vec3::new(0.0, -100.0, 0.0)));
}
