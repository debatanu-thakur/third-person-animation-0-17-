//! Pose extraction tool - Loads pose GLB files and saves as RON files

use bevy::prelude::*;
use bevy::gltf::Gltf;
use std::fs;
use std::path::Path;
use super::{Pose, BoneTransform, PoseMetadata, PoseId};

/// Resource to enable extraction mode
#[derive(Resource)]
pub struct ExtractionMode {
    pub enabled: bool,
    pub poses_input_path: String,
    pub ron_output_path: String,
}

impl Default for ExtractionMode {
    fn default() -> Self {
        Self {
            enabled: false,
            poses_input_path: "poses".to_string(),  // Relative to assets/
            ron_output_path: "assets/poses_ron".to_string(),
        }
    }
}

/// Configuration mapping GLB filenames to PoseIds
#[derive(Resource)]
pub struct ExtractionConfig {
    /// Map of (glb_filename, PoseId, notes)
    pub pose_mapping: Vec<PoseMapping>,
}

#[derive(Clone)]
pub struct PoseMapping {
    /// GLB filename (without .glb extension) in assets/poses/
    pub glb_name: String,
    /// Which pose this represents
    pub pose_id: PoseId,
    /// Optional notes
    pub notes: Option<String>,
}

/// Resource to track loaded pose GLBs
#[derive(Resource, Default)]
struct PoseGltfHandles {
    handles: Vec<(PoseId, Handle<Gltf>, String)>,  // (pose_id, handle, notes)
    loaded_count: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            pose_mapping: vec![
                PoseMapping {
                    glb_name: "idle".to_string(),
                    pose_id: PoseId::Idle,
                    notes: Some("Neutral standing pose".to_string()),
                },
                PoseMapping {
                    glb_name: "walk_left".to_string(),
                    pose_id: PoseId::WalkLeftFootForward,
                    notes: Some("Left foot forward, right foot back".to_string()),
                },
                PoseMapping {
                    glb_name: "walk_right".to_string(),
                    pose_id: PoseId::WalkRightFootForward,
                    notes: Some("Right foot forward, left foot back".to_string()),
                },
                PoseMapping {
                    glb_name: "run_left".to_string(),
                    pose_id: PoseId::RunLeftFootForward,
                    notes: Some("Running, left foot forward".to_string()),
                },
                PoseMapping {
                    glb_name: "run_right".to_string(),
                    pose_id: PoseId::RunRightFootForward,
                    notes: Some("Running, right foot forward".to_string()),
                },
                PoseMapping {
                    glb_name: "jump_takeoff".to_string(),
                    pose_id: PoseId::JumpTakeoff,
                    notes: Some("Crouch before jump".to_string()),
                },
                PoseMapping {
                    glb_name: "jump_air".to_string(),
                    pose_id: PoseId::JumpAirborne,
                    notes: Some("Mid-air pose".to_string()),
                },
                PoseMapping {
                    glb_name: "jump_land".to_string(),
                    pose_id: PoseId::JumpLanding,
                    notes: Some("Landing impact".to_string()),
                },
                PoseMapping {
                    glb_name: "roll_left".to_string(),
                    pose_id: PoseId::RollLeft,
                    notes: Some("Left roll".to_string()),
                },
                PoseMapping {
                    glb_name: "roll_right".to_string(),
                    pose_id: PoseId::RollRight,
                    notes: Some("Right roll".to_string()),
                },
                PoseMapping {
                    glb_name: "attack_punch".to_string(),
                    pose_id: PoseId::AttackPunch,
                    notes: Some("Punch pose".to_string()),
                },
                PoseMapping {
                    glb_name: "attack_kick".to_string(),
                    pose_id: PoseId::AttackKick,
                    notes: Some("Kick pose".to_string()),
                },
                PoseMapping {
                    glb_name: "crouch".to_string(),
                    pose_id: PoseId::Crouch,
                    notes: Some("Crouching pose".to_string()),
                },
            ],
        }
    }
}

/// Setup system that initializes extraction mode
pub fn setup_extraction_mode(
    mut commands: Commands,
) {
    // Only compile this code when extract_poses feature is enabled
    #[cfg(feature = "extract_poses")]
    {
        info!("🎬 Pose extraction mode ENABLED (feature flag active)");
        commands.insert_resource(ExtractionMode {
            enabled: true,
            ..default()
        });
        commands.insert_resource(ExtractionConfig::default());
    }

    #[cfg(not(feature = "extract_poses"))]
    {
        debug!("Pose extraction mode disabled. Run with: cargo run --features extract_poses");
    }
}

/// System to load pose GLB files
/// Only compiled when the extract_poses feature is enabled
#[cfg(feature = "extract_poses")]
pub fn load_pose_glbs(
    mut commands: Commands,
    extraction_mode: Option<Res<ExtractionMode>>,
    extraction_config: Option<Res<ExtractionConfig>>,
    asset_server: Res<AssetServer>,
    handles: Option<Res<PoseGltfHandles>>,
) {
    // Only run once
    if handles.is_some() {
        return;
    }

    let Some(mode) = extraction_mode else { return; };
    if !mode.enabled {
        return;
    }

    let Some(config) = extraction_config else { return; };

    info!("🎬 Loading {} pose GLB files from assets/{}/", config.pose_mapping.len(), mode.poses_input_path);

    let mut pose_handles = PoseGltfHandles::default();

    // Load each pose GLB
    for mapping in &config.pose_mapping {
        let glb_path = format!("{}/{}.glb", mode.poses_input_path, mapping.glb_name);
        let handle: Handle<Gltf> = asset_server.load(&glb_path);

        pose_handles.handles.push((
            mapping.pose_id,
            handle,
            mapping.notes.clone().unwrap_or_default(),
        ));

        info!("  Loading: {} → {:?}", glb_path, mapping.pose_id);
    }

    commands.insert_resource(pose_handles);
}

/// Dummy system when extract_poses feature is not enabled
#[cfg(not(feature = "extract_poses"))]
pub fn load_pose_glbs() {}

/// System to extract poses from loaded GLBs
/// Only compiled when the extract_poses feature is enabled
#[cfg(feature = "extract_poses")]
pub fn extract_poses_from_glbs(
    mut commands: Commands,
    extraction_mode: Option<Res<ExtractionMode>>,
    mut pose_handles: Option<ResMut<PoseGltfHandles>>,
    gltf_assets: Res<Assets<Gltf>>,
    scenes: Res<Assets<Scene>>,
    mut extracted: Local<bool>,
) {
    if *extracted {
        return;
    }

    let Some(mode) = extraction_mode else { return; };
    let Some(ref mut handles) = pose_handles else { return; };

    // Check if all GLTFs are loaded
    let total_count = handles.handles.len();
    let loaded_count = handles.handles.iter()
        .filter(|(_, handle, _)| gltf_assets.get(handle).is_some())
        .count();

    if loaded_count < total_count {
        if loaded_count != handles.loaded_count {
            info!("Loading poses: {}/{}", loaded_count, total_count);
            handles.loaded_count = loaded_count;
        }
        return;
    }

    info!("✓ All {} pose GLBs loaded. Starting extraction...", total_count);

    // Create output directory
    let output_path = Path::new(&mode.ron_output_path);
    if !output_path.exists() {
        fs::create_dir_all(output_path)
            .unwrap_or_else(|e| error!("Failed to create output directory: {}", e));
    }

    // Extract each pose
    for (pose_id, gltf_handle, notes) in &handles.handles {
        if let Some(gltf) = gltf_assets.get(gltf_handle) {
            match extract_pose_from_gltf(gltf, &scenes, *pose_id, notes.clone()) {
                Ok(pose) => {
                    save_pose_to_ron(&pose, *pose_id, output_path);
                }
                Err(e) => {
                    error!("Failed to extract pose {:?}: {}", pose_id, e);
                }
            }
        }
    }

    info!("✅ Pose extraction complete! Check {}", mode.ron_output_path);
    *extracted = true;

    // Clean up
    commands.remove_resource::<PoseGltfHandles>();
    commands.remove_resource::<ExtractionMode>();
}

/// Dummy system when extract_poses feature is not enabled
#[cfg(not(feature = "extract_poses"))]
pub fn extract_poses_from_glbs() {}

/// Extract a single pose from a loaded GLTF
#[cfg(feature = "extract_poses")]
fn extract_pose_from_gltf(
    gltf: &Gltf,
    scenes: &Assets<Scene>,
    pose_id: PoseId,
    notes: String,
) -> Result<Pose, String> {
    let mut pose = Pose::new(pose_id.name());

    // Get the first scene from the GLTF
    let scene_handle = gltf.scenes.first()
        .ok_or("No scenes found in GLTF")?;

    let scene = scenes.get(scene_handle)
        .ok_or("Scene not loaded")?;

    // Extract bone transforms from the scene's world
    let world = &scene.world;

    let mut bone_count = 0;

    // Query all entities with Transform and Name components
    let mut query = world.query::<(&Transform, &Name)>();

    for (transform, name) in query.iter(world) {
        let bone_name = name.as_str().to_string();

        // Store the transform
        pose.bone_transforms.insert(
            bone_name.clone(),
            BoneTransform::from(*transform),
        );

        bone_count += 1;
    }

    pose.metadata = PoseMetadata {
        source_animation: Some(format!("{:?}.glb", pose_id)),
        source_frame: None,
        source_time: None,
        notes: Some(notes),
    };

    info!("✓ Extracted pose '{}' with {} bones", pose_id.name(), bone_count);

    if bone_count == 0 {
        warn!("⚠️  No bones found in pose '{}'", pose_id.name());
    }

    Ok(pose)
}

/// Save a pose to a RON file
#[cfg(feature = "extract_poses")]
fn save_pose_to_ron(pose: &Pose, pose_id: PoseId, output_path: &Path) {
    let filename = format!("{}.pose.ron", pose_id_to_filename(pose_id));
    let filepath = output_path.join(filename);

    match ron::ser::to_string_pretty(pose, ron::ser::PrettyConfig::default()) {
        Ok(ron_string) => {
            match fs::write(&filepath, ron_string) {
                Ok(_) => info!("✓ Saved pose to {}", filepath.display()),
                Err(e) => error!("Failed to write pose file: {}", e),
            }
        }
        Err(e) => error!("Failed to serialize pose to RON: {}", e),
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
