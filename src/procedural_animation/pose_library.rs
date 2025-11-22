//! Pose library management for the 13 keyframe poses

use bevy::prelude::*;
use super::{Pose, PoseId};
use std::collections::HashMap;

/// Resource containing all 13 keyframe poses
#[derive(Resource, Default)]
pub struct PoseLibrary {
    poses: HashMap<PoseId, Handle<Pose>>,
}

impl PoseLibrary {
    /// Create a new empty pose library
    pub fn new() -> Self {
        Self {
            poses: HashMap::new(),
        }
    }

    /// Add a pose to the library
    pub fn add_pose(&mut self, pose_id: PoseId, handle: Handle<Pose>) {
        self.poses.insert(pose_id, handle);
    }

    /// Get a pose handle by ID
    pub fn get(&self, pose_id: PoseId) -> Option<&Handle<Pose>> {
        self.poses.get(&pose_id)
    }

    /// Check if library has all 13 poses loaded
    pub fn is_complete(&self) -> bool {
        self.poses.len() == 13
    }

    /// Get list of missing poses
    pub fn missing_poses(&self) -> Vec<PoseId> {
        PoseId::all()
            .iter()
            .filter(|id| !self.poses.contains_key(id))
            .copied()
            .collect()
    }
}

/// System to load pose assets from RON files
pub fn load_pose_library(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut library = PoseLibrary::new();

    // Load each pose from assets/poses/ directory
    for pose_id in PoseId::all() {
        let path = format!("poses/{}.pose.ron", pose_id_to_filename(pose_id));
        let handle: Handle<Pose> = asset_server.load(&path);
        library.add_pose(pose_id, handle);
        info!("Loading pose: {} from {}", pose_id.name(), path);
    }

    commands.insert_resource(library);
}

/// System to check if all poses are loaded
pub fn check_pose_loading(
    library: Option<Res<PoseLibrary>>,
    pose_assets: Res<Assets<Pose>>,
) {
    let Some(library) = library else {
        return;
    };

    if !library.is_complete() {
        warn!("Pose library incomplete. Missing: {:?}", library.missing_poses());
        return;
    }

    // Check if all assets are actually loaded
    let mut loaded_count = 0;
    for pose_id in PoseId::all() {
        if let Some(handle) = library.get(pose_id) {
            if pose_assets.get(handle).is_some() {
                loaded_count += 1;
            }
        }
    }

    if loaded_count == 13 {
        info!("✓ All 13 poses loaded successfully!");
    } else {
        debug!("Pose loading progress: {}/13", loaded_count);
    }
}

/// Convert pose ID to filename
fn pose_id_to_filename(pose_id: PoseId) -> &'static str {
    match pose_id {
        PoseId::Idle => "idle",
        PoseId::WalkLeftFootForward => "walk_left",
        PoseId::WalkRightFootForward => "walk_right",
        PoseId::RunLeftFootForward => "run_left",
        PoseId::RunRightFootForward => "run_right",
        PoseId::JumpTakeoff => "jump_takeoff",
        PoseId::JumpAirborne => "jump_airborne",
        PoseId::JumpLanding => "jump_landing",
        PoseId::RollLeft => "roll_left",
        PoseId::RollRight => "roll_right",
        PoseId::AttackPunch => "attack_punch",
        PoseId::AttackKick => "attack_kick",
        PoseId::Crouch => "crouch",
    }
}
