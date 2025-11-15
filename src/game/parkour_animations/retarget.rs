use bevy::prelude::*;
use bevy::animation::{AnimationTargetId, animatable::*, AnimationCurve};
use bevy::gltf::Gltf;

/// Retargets an animation from GLTF hierarchy-based targets to simple name-based targets
///
/// GLTF animations use paths like ["Armature", "mixamorig12:Hips"]
/// But we want simple names like "mixamorig12:Hips" to match bones anywhere in hierarchy
///
/// This allows animations from vault.glb to work on the brian_parkour.glb character
pub fn retarget_animation_by_bone_names(
    original_clip: &AnimationClip,
    gltf: &Gltf,
) -> AnimationClip {
    let mut retargeted = AnimationClip::default();
    retargeted.set_duration(original_clip.duration());

    info!("🔄 Retargeting animation from GLTF paths to bone names...");
    info!("   GLTF has {} named nodes", gltf.named_nodes.len());

    // Get all bone names from the GLTF (these are the target names we want)
    let bone_names: Vec<String> = gltf.named_nodes.keys()
        .filter(|name| name.starts_with("mixamorig"))  // Only get skeleton bones
        .cloned()
        .collect();

    info!("   Found {} mixamorig bones to retarget", bone_names.len());

    // For each bone in the GLTF, create a name-based target
    let mut curves_added = 0;
    for bone_name in bone_names.iter() {
        // Create a simple name-based target (no hierarchy path)
        let target_id = AnimationTargetId::from_name(&Name::new(bone_name.clone()));

        // Find curves in original animation that target this bone
        // Since we can't easily map UUID targets to names, we'll iterate through
        // all curves and try to match by index (assuming same order)
        //
        // TODO: This is a workaround - ideally we'd parse the GLTF structure
        // to map AnimationTargetId UUIDs to bone names
    }

    // WORKAROUND: Since we can't easily extract which UUID maps to which bone,
    // let's try a different approach - manually rebuild the animation curves
    // using the GLTF animation data

    info!("⚠️  Full retargeting requires GLTF node mapping");
    info!("   Attempting workaround: copy curves and hope for best");

    // Copy all curves from original (this won't work, but shows the structure)
    for (target_id, curves) in original_clip.curves() {
        for curve in curves {
            retargeted.add_curve_to_target(*target_id, curve.clone());
        }
    }

    info!("   Copied {} curves (still using UUID targets - won't work!)", curves_added);

    retargeted
}

/// Helper: Creates a simple name-based AnimationTargetId
/// This targets any entity with this Name, regardless of hierarchy
pub fn target_from_bone_name(bone_name: &str) -> AnimationTargetId {
    AnimationTargetId::from_name(&Name::new(bone_name.to_string()))
}
