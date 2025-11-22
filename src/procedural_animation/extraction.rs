//! Pose extraction tool - Samples frames from GLB animations and saves as RON files

use bevy::prelude::*;
use bevy::gltf::Gltf;
use std::fs;
use std::path::Path;
use super::{Pose, BoneTransform, PoseMetadata, PoseId};

/// Resource to enable extraction mode
#[derive(Resource)]
pub struct ExtractionMode {
    pub enabled: bool,
    pub output_path: String,
}

impl Default for ExtractionMode {
    fn default() -> Self {
        Self {
            enabled: false,
            output_path: "assets/poses".to_string(),
        }
    }
}

/// Configuration for which frames to extract from each animation
#[derive(Resource)]
pub struct ExtractionConfig {
    /// Map of (animation_name, frame_time) -> PoseId
    pub extraction_map: Vec<ExtractionEntry>,
}

#[derive(Clone)]
pub struct ExtractionEntry {
    pub animation_name: String,
    pub time_seconds: f32,
    pub pose_id: PoseId,
    pub notes: Option<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            extraction_map: vec![
                // Idle pose
                ExtractionEntry {
                    animation_name: "idle".to_string(),
                    time_seconds: 0.5,
                    pose_id: PoseId::Idle,
                    notes: Some("Neutral standing pose".to_string()),
                },

                // Walk cycle - left foot forward (mid-stride)
                ExtractionEntry {
                    animation_name: "walk".to_string(),
                    time_seconds: 0.25, // Quarter way through cycle
                    pose_id: PoseId::WalkLeftFootForward,
                    notes: Some("Left foot forward, right foot back".to_string()),
                },

                // Walk cycle - right foot forward
                ExtractionEntry {
                    animation_name: "walk".to_string(),
                    time_seconds: 0.75, // Three quarters through cycle
                    pose_id: PoseId::WalkRightFootForward,
                    notes: Some("Right foot forward, left foot back".to_string()),
                },

                // Run cycle - left foot forward
                ExtractionEntry {
                    animation_name: "running".to_string(),
                    time_seconds: 0.2,
                    pose_id: PoseId::RunLeftFootForward,
                    notes: Some("Left foot forward, right foot back, running pose".to_string()),
                },

                // Run cycle - right foot forward
                ExtractionEntry {
                    animation_name: "running".to_string(),
                    time_seconds: 0.6,
                    pose_id: PoseId::RunRightFootForward,
                    notes: Some("Right foot forward, left foot back, running pose".to_string()),
                },

                // Jump takeoff
                ExtractionEntry {
                    animation_name: "standing_jump".to_string(),
                    time_seconds: 0.1,
                    pose_id: PoseId::JumpTakeoff,
                    notes: Some("Crouch before jump".to_string()),
                },

                // Jump airborne
                ExtractionEntry {
                    animation_name: "standing_jump".to_string(),
                    time_seconds: 0.5,
                    pose_id: PoseId::JumpAirborne,
                    notes: Some("Mid-air pose".to_string()),
                },

                // Jump landing
                ExtractionEntry {
                    animation_name: "standing_jump".to_string(),
                    time_seconds: 0.9,
                    pose_id: PoseId::JumpLanding,
                    notes: Some("Landing crouch".to_string()),
                },

                // TODO: Add extraction entries for rolls and attacks
                // These will need appropriate animations in the GLB

                // Placeholders for now (using idle)
                ExtractionEntry {
                    animation_name: "idle".to_string(),
                    time_seconds: 0.0,
                    pose_id: PoseId::RollLeft,
                    notes: Some("PLACEHOLDER - needs roll animation".to_string()),
                },
                ExtractionEntry {
                    animation_name: "idle".to_string(),
                    time_seconds: 0.0,
                    pose_id: PoseId::RollRight,
                    notes: Some("PLACEHOLDER - needs roll animation".to_string()),
                },
                ExtractionEntry {
                    animation_name: "idle".to_string(),
                    time_seconds: 0.0,
                    pose_id: PoseId::AttackPunch,
                    notes: Some("PLACEHOLDER - needs attack animation".to_string()),
                },
                ExtractionEntry {
                    animation_name: "idle".to_string(),
                    time_seconds: 0.0,
                    pose_id: PoseId::AttackKick,
                    notes: Some("PLACEHOLDER - needs attack animation".to_string()),
                },
                ExtractionEntry {
                    animation_name: "idle".to_string(),
                    time_seconds: 0.3,
                    pose_id: PoseId::Crouch,
                    notes: Some("Slight crouch from idle".to_string()),
                },
            ],
        }
    }
}

/// Setup system that initializes extraction mode
pub fn setup_extraction_mode(
    mut commands: Commands,
) {
    // Check environment variable to enable extraction mode
    let extraction_enabled = std::env::var("EXTRACT_POSES").is_ok();

    if extraction_enabled {
        info!("🎬 Pose extraction mode ENABLED");
        commands.insert_resource(ExtractionMode {
            enabled: true,
            ..default()
        });
        commands.insert_resource(ExtractionConfig::default());
    } else {
        debug!("Pose extraction mode disabled. Set EXTRACT_POSES=1 to enable");
    }
}

/// System to perform pose extraction from loaded animations
pub fn extract_poses_from_animations(
    mut commands: Commands,
    extraction_mode: Option<Res<ExtractionMode>>,
    extraction_config: Option<Res<ExtractionConfig>>,
    gltf_asset: Res<crate::game::player::assets::PlayerGltfAsset>,
    gltf_assets: Res<Assets<Gltf>>,
    animation_clips: Res<Assets<AnimationClip>>,
    mut extracted: Local<bool>,
) {
    // Only run if extraction mode is enabled
    let Some(mode) = extraction_mode else { return; };
    if !mode.enabled || *extracted {
        return;
    }

    let Some(config) = extraction_config else { return; };

    // Wait for GLTF to load
    let Some(gltf) = gltf_assets.get(&gltf_asset.gltf) else {
        return;
    };

    info!("🎬 Starting pose extraction from {} animations", gltf.named_animations.len());

    // Create output directory
    let output_path = Path::new(&mode.output_path);
    if !output_path.exists() {
        fs::create_dir_all(output_path)
            .unwrap_or_else(|e| error!("Failed to create poses directory: {}", e));
    }

    // Extract each configured pose
    for entry in &config.extraction_map {
        if let Some(anim_handle) = gltf.named_animations.get(entry.animation_name.as_str()) {
            if let Some(animation_clip) = animation_clips.get(anim_handle) {
                match extract_pose_at_time(
                    animation_clip,
                    entry.time_seconds,
                    &entry.animation_name,
                    entry.pose_id,
                    entry.notes.clone(),
                ) {
                    Ok(pose) => {
                        // Save pose to RON file
                        save_pose_to_ron(&pose, entry.pose_id, output_path);
                    }
                    Err(e) => {
                        error!("Failed to extract pose {:?}: {}", entry.pose_id, e);
                    }
                }
            } else {
                warn!("Animation clip not loaded yet for '{}'", entry.animation_name);
            }
        } else {
            warn!("Animation '{}' not found in GLTF", entry.animation_name);
        }
    }

    info!("✅ Pose extraction complete! Check {}", mode.output_path);
    *extracted = true;

    // Disable extraction mode to prevent re-running
    commands.remove_resource::<ExtractionMode>();
}

/// Extract a single pose from an animation at a specific time
fn extract_pose_at_time(
    animation_clip: &AnimationClip,
    time_seconds: f32,
    source_animation: &str,
    pose_id: PoseId,
    notes: Option<String>,
) -> Result<Pose, String> {
    let mut pose = Pose::new(pose_id.name());

    pose.metadata = PoseMetadata {
        source_animation: Some(source_animation.to_string()),
        source_time: Some(time_seconds),
        source_frame: None,
        notes,
    };

    // Iterate through all curves in the animation
    for (target_id, curves) in animation_clip.curves() {
        // For each target (bone), sample its transform at the given time
        // Note: This is simplified - in reality we need to sample the curves
        // and construct the transform from rotation/translation/scale curves

        // TODO: Properly sample the curves using curve.sample_clamped(time_seconds)
        // For now, we'll add a placeholder transform

        // The target_id contains the bone name/path
        let bone_name = format!("{:?}", target_id); // Simplified - need better name extraction

        // Sample each curve for this target
        // The curves contain rotation, translation, scale data
        // We need to sample all three and combine into a Transform

        warn!("TODO: Implement proper curve sampling for bone: {}", bone_name);

        // Placeholder transform
        pose.bone_transforms.insert(
            bone_name,
            BoneTransform {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        );
    }

    info!("✓ Extracted pose '{}' from '{}' at {}s ({} bones)",
        pose_id.name(),
        source_animation,
        time_seconds,
        pose.bone_transforms.len()
    );

    Ok(pose)
}

/// Save a pose to a RON file
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
