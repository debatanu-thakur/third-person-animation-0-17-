use bevy::{prelude::*, gltf::Gltf, animation::AnimationClip};
use std::collections::HashMap;
use crate::screens::Screen;

// ============================================================================
// PARKOUR ANIMATION LIBRARY
// ============================================================================

/// Resource holding handles to animation-only GLB files
#[derive(Resource)]
pub struct ParkourAnimationLibrary {
    /// GLB file handles (contain animations)
    pub vault_gltf: Handle<Gltf>,
    pub climb_gltf: Handle<Gltf>,
    pub slide_gltf: Handle<Gltf>,
    pub wall_run_left_gltf: Handle<Gltf>,
    pub wall_run_right_gltf: Handle<Gltf>,
    pub roll_gltf: Handle<Gltf>,

    /// Extracted animation clips (set after GLBs load)
    pub vault: Option<Handle<AnimationClip>>,
    pub climb: Option<Handle<AnimationClip>>,
    pub slide: Option<Handle<AnimationClip>>,
    pub wall_run_left: Option<Handle<AnimationClip>>,
    pub wall_run_right: Option<Handle<AnimationClip>>,
    pub roll: Option<Handle<AnimationClip>>,

    /// Loaded flag
    pub loaded: bool,
}

impl FromWorld for ParkourAnimationLibrary {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            // Load animation-only GLB files
            vault_gltf: asset_server.load("parkour_animations/vault.glb"),
            climb_gltf: asset_server.load("parkour_animations/climb.glb"),
            slide_gltf: asset_server.load("parkour_animations/slide.glb"),
            wall_run_left_gltf: asset_server.load("parkour_animations/wall_run_left.glb"),
            wall_run_right_gltf: asset_server.load("parkour_animations/wall_run_right.glb"),
            roll_gltf: asset_server.load("parkour_animations/roll.glb"),

            vault: None,
            climb: None,
            slide: None,
            wall_run_left: None,
            wall_run_right: None,
            roll: None,

            loaded: false,
        }
    }
}

// ============================================================================
// BONE NAME MAPPING
// ============================================================================

/// Stores bone names found in animation files for verification
#[derive(Resource, Default)]
pub struct AnimationBoneNames {
    pub character_bones: Vec<String>,
    pub animation_bones: HashMap<String, Vec<String>>, // animation_name -> bone_names
    pub verified: bool,
}

/// Sampled bone transform at a specific time
#[derive(Debug, Clone)]
pub struct SampledBoneTransform {
    pub bone_name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub time: f32,
}

/// Keyframe data extracted from animation
#[derive(Debug, Clone)]
pub struct AnimationKeyframe {
    pub time: f32,
    pub bones: Vec<SampledBoneTransform>,
}

// ============================================================================
// ANIMATION EXTRACTION SYSTEM
// ============================================================================

/// Extracts animation clips from loaded GLB files
pub fn extract_parkour_animations(
    mut library: ResMut<ParkourAnimationLibrary>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    if library.loaded {
        return;
    }

    // Try to extract animations from each GLB
    let mut extracted_count = 0;

    // Vault
    if library.vault.is_none() {
        if let Some(gltf) = gltf_assets.get(&library.vault_gltf) {
            if let Some(animation) = gltf.animations.first() {
                library.vault = Some(animation.clone());
                info!("✅ Loaded vault animation");
                extracted_count += 1;
            }
        }
    }

    // Climb
    if library.climb.is_none() {
        if let Some(gltf) = gltf_assets.get(&library.climb_gltf) {
            if let Some(animation) = gltf.animations.first() {
                library.climb = Some(animation.clone());
                info!("✅ Loaded climb animation");
                extracted_count += 1;
            }
        }
    }

    // Slide
    if library.slide.is_none() {
        if let Some(gltf) = gltf_assets.get(&library.slide_gltf) {
            if let Some(animation) = gltf.animations.first() {
                library.slide = Some(animation.clone());
                info!("✅ Loaded slide animation");
                extracted_count += 1;
            }
        }
    }

    // Wall Run Left
    if library.wall_run_left.is_none() {
        if let Some(gltf) = gltf_assets.get(&library.wall_run_left_gltf) {
            if let Some(animation) = gltf.animations.first() {
                library.wall_run_left = Some(animation.clone());
                info!("✅ Loaded wall_run_left animation");
                extracted_count += 1;
            }
        }
    }

    // Wall Run Right
    if library.wall_run_right.is_none() {
        if let Some(gltf) = gltf_assets.get(&library.wall_run_right_gltf) {
            if let Some(animation) = gltf.animations.first() {
                library.wall_run_right = Some(animation.clone());
                info!("✅ Loaded wall_run_right animation");
                extracted_count += 1;
            }
        }
    }

    // Roll
    if library.roll.is_none() {
        if let Some(gltf) = gltf_assets.get(&library.roll_gltf) {
            if let Some(animation) = gltf.animations.first() {
                library.roll = Some(animation.clone());
                info!("✅ Loaded roll animation");
                extracted_count += 1;
            }
        }
    }

    // Check if all loaded
    if library.vault.is_some()
        && library.climb.is_some()
        && library.slide.is_some()
        && library.wall_run_left.is_some()
        && library.wall_run_right.is_some()
        && library.roll.is_some()
    {
        library.loaded = true;
        info!("🎉 All parkour animations loaded successfully!");
    }
}

// ============================================================================
// BONE NAME COLLECTION SYSTEM
// ============================================================================

/// Collects bone names from character rig for verification
pub fn collect_character_bone_names(
    mut bone_names: ResMut<AnimationBoneNames>,
    bone_query: Query<&Name, Added<Name>>,
) {
    if bone_names.verified {
        return;
    }

    for name in bone_query.iter() {
        let bone_name = name.as_str();

        // Only collect Mixamo rig bones
        if bone_name.starts_with("mixamorig:") {
            if !bone_names.character_bones.contains(&bone_name.to_string()) {
                bone_names.character_bones.push(bone_name.to_string());
            }
        }
    }

    // Log when we have a good collection
    if bone_names.character_bones.len() > 20 && !bone_names.verified {
        info!("📋 Collected {} character bones:", bone_names.character_bones.len());
        info!("   Sample bones: {:?}", &bone_names.character_bones[..5.min(bone_names.character_bones.len())]);
    }
}

/// Collects bone names from animation clips
pub fn collect_animation_bone_names(
    library: Res<ParkourAnimationLibrary>,
    animation_clips: Res<Assets<AnimationClip>>,
    mut bone_names: ResMut<AnimationBoneNames>,
) {
    if !library.loaded || bone_names.verified {
        return;
    }

    // Check vault animation bones
    if let Some(vault_handle) = &library.vault {
        if let Some(clip) = animation_clips.get(vault_handle) {
            if !bone_names.animation_bones.contains_key("vault") {
                let bones = extract_bone_names_from_clip(clip);
                bone_names.animation_bones.insert("vault".to_string(), bones);
                info!("📋 Collected bone names from vault animation");
            }
        }
    }

    // Verify bone matching
    if !bone_names.animation_bones.is_empty() && !bone_names.character_bones.is_empty() {
        verify_bone_matching(&bone_names);
        bone_names.verified = true;
    }
}

/// Extract bone names from an animation clip
fn extract_bone_names_from_clip(clip: &AnimationClip) -> Vec<String> {
    let mut bone_names = Vec::new();

    for (target_id, _curves) in clip.curves() {
        // EntityPath contains bone hierarchy like ["mixamorig:Hips", "mixamorig:Spine"]
        if let Some(last_part) = target_id.parts().last() {
            let bone_name = last_part.to_string();
            if !bone_names.contains(&bone_name) {
                bone_names.push(bone_name);
            }
        }
    }

    bone_names
}

/// Verify that animation bones match character bones
fn verify_bone_matching(bone_names: &AnimationBoneNames) {
    info!("🔍 Verifying bone name matching...");

    for (anim_name, anim_bones) in &bone_names.animation_bones {
        let mut matched = 0;
        let mut missing = Vec::new();

        for anim_bone in anim_bones {
            if bone_names.character_bones.contains(anim_bone) {
                matched += 1;
            } else {
                missing.push(anim_bone.clone());
            }
        }

        let match_percent = (matched as f32 / anim_bones.len() as f32) * 100.0;

        if match_percent > 90.0 {
            info!("✅ {}: {}/{} bones matched ({:.1}%)",
                anim_name, matched, anim_bones.len(), match_percent);
        } else {
            warn!("⚠️  {}: Only {}/{} bones matched ({:.1}%)",
                anim_name, matched, anim_bones.len(), match_percent);
            if !missing.is_empty() {
                warn!("   Missing bones: {:?}", &missing[..5.min(missing.len())]);
            }
        }
    }
}

// ============================================================================
// ANIMATION SAMPLING
// ============================================================================

/// Sample an animation clip at a specific time
pub fn sample_animation_at_time(
    clip: &AnimationClip,
    time: f32,
) -> Vec<SampledBoneTransform> {
    let mut samples = Vec::new();

    for (target_id, curves) in clip.curves() {
        let bone_name = target_id.parts()
            .last()
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Sample translation curve
        let translation = if let Some(curve) = curves.translation() {
            sample_vec3_curve(curve, time)
        } else {
            Vec3::ZERO
        };

        // Sample rotation curve
        let rotation = if let Some(curve) = curves.rotation() {
            sample_quat_curve(curve, time)
        } else {
            Quat::IDENTITY
        };

        samples.push(SampledBoneTransform {
            bone_name,
            translation,
            rotation,
            time,
        });
    }

    samples
}

/// Sample a Vec3 animation curve at a specific time
fn sample_vec3_curve(curve: &bevy::animation::AnimationCurve<Vec3>, time: f32) -> Vec3 {
    // Find keyframes before and after the target time
    let keyframes = curve.keyframes();

    if keyframes.is_empty() {
        return Vec3::ZERO;
    }

    // Simple linear interpolation between keyframes
    // TODO: Use actual curve interpolation mode
    for i in 0..keyframes.len() - 1 {
        let k1 = &keyframes[i];
        let k2 = &keyframes[i + 1];

        if time >= k1.0 && time <= k2.0 {
            let t = (time - k1.0) / (k2.0 - k1.0);
            return k1.1.lerp(k2.1, t);
        }
    }

    // Return last keyframe if time is beyond
    keyframes.last().map(|k| k.1).unwrap_or(Vec3::ZERO)
}

/// Sample a Quat animation curve at a specific time
fn sample_quat_curve(curve: &bevy::animation::AnimationCurve<Quat>, time: f32) -> Quat {
    let keyframes = curve.keyframes();

    if keyframes.is_empty() {
        return Quat::IDENTITY;
    }

    for i in 0..keyframes.len() - 1 {
        let k1 = &keyframes[i];
        let k2 = &keyframes[i + 1];

        if time >= k1.0 && time <= k2.0 {
            let t = (time - k1.0) / (k2.0 - k1.0);
            return k1.1.slerp(k2.1, t);
        }
    }

    keyframes.last().map(|k| k.1).unwrap_or(Quat::IDENTITY)
}

// ============================================================================
// DEBUG: SAMPLE AND PRINT
// ============================================================================

/// Debug system to sample animation on key press
pub fn debug_sample_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Res<ParkourAnimationLibrary>,
    animation_clips: Res<Assets<AnimationClip>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

    if !library.loaded {
        warn!("Parkour animations not loaded yet!");
        return;
    }

    // Sample vault animation at 0.5 seconds
    if let Some(vault_handle) = &library.vault {
        if let Some(clip) = animation_clips.get(vault_handle) {
            let samples = sample_animation_at_time(clip, 0.5);

            info!("📊 Sampled vault animation at 0.5s:");
            for sample in samples.iter().take(5) {
                info!("   {} → pos: {:.2?}, rot: {:.2?}",
                    sample.bone_name, sample.translation, sample.rotation);
            }
            info!("   ... and {} more bones", samples.len().saturating_sub(5));
        }
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourAnimationLibrary>();
    app.init_resource::<AnimationBoneNames>();

    app.add_systems(
        Update,
        (
            extract_parkour_animations,
            collect_character_bone_names,
            collect_animation_bone_names,
            debug_sample_animation,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
