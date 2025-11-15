use bevy::{prelude::*, gltf::GltfAssetLabel};
use std::collections::HashMap;
use crate::screens::Screen;

// ============================================================================
// PARKOUR ANIMATION LIBRARY
// ============================================================================

/// Resource holding animation graphs for parkour animations
#[derive(Resource)]
pub struct ParkourAnimationLibrary {
    /// Animation clip handles loaded from GLB files
    pub vault_clip: Handle<AnimationClip>,
    pub climb_clip: Handle<AnimationClip>,
    pub slide_clip: Handle<AnimationClip>,
    pub wall_run_left_clip: Handle<AnimationClip>,
    pub wall_run_right_clip: Handle<AnimationClip>,
    pub roll_clip: Handle<AnimationClip>,

    /// Animation graphs (created from clips)
    pub vault_graph: Handle<AnimationGraph>,
    pub climb_graph: Handle<AnimationGraph>,
    pub slide_graph: Handle<AnimationGraph>,
    pub wall_run_left_graph: Handle<AnimationGraph>,
    pub wall_run_right_graph: Handle<AnimationGraph>,
    pub roll_graph: Handle<AnimationGraph>,

    /// Animation node indices (for playing)
    pub vault_node: AnimationNodeIndex,
    pub climb_node: AnimationNodeIndex,
    pub slide_node: AnimationNodeIndex,
    pub wall_run_left_node: AnimationNodeIndex,
    pub wall_run_right_node: AnimationNodeIndex,
    pub roll_node: AnimationNodeIndex,

    /// Loaded flag
    pub loaded: bool,
}

impl FromWorld for ParkourAnimationLibrary {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        // Load animation clips directly from GLB files using GltfAssetLabel
        // Animation index 0 means the first animation in the GLB file
        let vault_clip = asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("models/animations/vault_over_rining.glb")
        );
        let climb_clip = asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("models/animations/Freehang Climb.glb")
        );
        let slide_clip = asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("models/animations/Running Slide.glb")
        );
        let wall_run_left_clip = asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("models/animations/Over Obstacle Jumping.glb")
        );
        let wall_run_right_clip = asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("models/animations/Over Obstacle Jumping.glb")
        );
        let roll_clip = asset_server.load(
            GltfAssetLabel::Animation(0).from_asset("models/animations/Falling To Roll.glb")
        );

        // Create AnimationGraphs from clips
        let (vault_graph, vault_node) = AnimationGraph::from_clip(vault_clip.clone());
        let (climb_graph, climb_node) = AnimationGraph::from_clip(climb_clip.clone());
        let (slide_graph, slide_node) = AnimationGraph::from_clip(slide_clip.clone());
        let (wall_run_left_graph, wall_run_left_node) = AnimationGraph::from_clip(wall_run_left_clip.clone());
        let (wall_run_right_graph, wall_run_right_node) = AnimationGraph::from_clip(wall_run_right_clip.clone());
        let (roll_graph, roll_node) = AnimationGraph::from_clip(roll_clip.clone());

        // Add graphs to assets
        let mut animation_graphs = world.resource_mut::<Assets<AnimationGraph>>();
        let vault_graph = animation_graphs.add(vault_graph);
        let climb_graph = animation_graphs.add(climb_graph);
        let slide_graph = animation_graphs.add(slide_graph);
        let wall_run_left_graph = animation_graphs.add(wall_run_left_graph);
        let wall_run_right_graph = animation_graphs.add(wall_run_right_graph);
        let roll_graph = animation_graphs.add(roll_graph);

        Self {
            vault_clip,
            climb_clip,
            slide_clip,
            wall_run_left_clip,
            wall_run_right_clip,
            roll_clip,

            vault_graph,
            climb_graph,
            slide_graph,
            wall_run_left_graph,
            wall_run_right_graph,
            roll_graph,

            vault_node,
            climb_node,
            slide_node,
            wall_run_left_node,
            wall_run_right_node,
            roll_node,

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
// ANIMATION LOADING CHECK SYSTEM
// ============================================================================

/// Checks if animation clips have loaded
pub fn check_parkour_animations_loaded(
    mut library: ResMut<ParkourAnimationLibrary>,
    animation_clips: Res<Assets<AnimationClip>>,
) {
    if library.loaded {
        return;
    }

    // Check if all animation clips are loaded
    let vault_loaded = animation_clips.get(&library.vault_clip).is_some();
    let climb_loaded = animation_clips.get(&library.climb_clip).is_some();
    let slide_loaded = animation_clips.get(&library.slide_clip).is_some();
    let wall_run_left_loaded = animation_clips.get(&library.wall_run_left_clip).is_some();
    let wall_run_right_loaded = animation_clips.get(&library.wall_run_right_clip).is_some();
    let roll_loaded = animation_clips.get(&library.roll_clip).is_some();

    if vault_loaded && climb_loaded && slide_loaded
        && wall_run_left_loaded && wall_run_right_loaded && roll_loaded
    {
        library.loaded = true;
        info!("🎉 All parkour animations loaded successfully!");
        info!("   - vault: {}", if vault_loaded { "✅" } else { "❌" });
        info!("   - climb: {}", if climb_loaded { "✅" } else { "❌" });
        info!("   - slide: {}", if slide_loaded { "✅" } else { "❌" });
        info!("   - wall_run_left: {}", if wall_run_left_loaded { "✅" } else { "❌" });
        info!("   - wall_run_right: {}", if wall_run_right_loaded { "✅" } else { "❌" });
        info!("   - roll: {}", if roll_loaded { "✅" } else { "❌" });
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
        if bone_name.starts_with("mixamorig12:") {
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
    if let Some(clip) = animation_clips.get(&library.vault_clip) {
        if !bone_names.animation_bones.contains_key("vault") {
            let bones = extract_bone_names_from_clip(clip);
            bone_names.animation_bones.insert("vault".to_string(), bones);
            info!("📋 Collected bone names from vault animation");
        }
    }

    // Verify bone matching
    if !bone_names.animation_bones.is_empty() && !bone_names.character_bones.is_empty() {
        verify_bone_matching(&bone_names);
        bone_names.verified = true;
    }
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
// ANIMATION SAMPLING RESOURCES
// ============================================================================

/// Stores sampled animation poses for IK targeting
#[derive(Resource, Default)]
pub struct SampledParkourPoses {
    /// Vault animation samples at key times (0.0s, 0.25s, 0.5s, 0.75s, 1.0s)
    pub vault_samples: HashMap<String, Vec<SampledBoneTransform>>, // time_key -> bone_transforms

    /// Climb animation samples
    pub climb_samples: HashMap<String, Vec<SampledBoneTransform>>,

    /// Slide animation samples
    pub slide_samples: HashMap<String, Vec<SampledBoneTransform>>,

    /// Whether sampling is complete
    pub sampled: bool,
}

impl SampledParkourPoses {
    /// Get hand position from vault animation at specific time
    pub fn get_vault_hand_pos(&self, time: f32, hand: &str) -> Option<Vec3> {
        let time_key = format!("{:.2}", time);
        if let Some(bones) = self.vault_samples.get(&time_key) {
            for bone in bones {
                if bone.bone_name.contains(hand) {
                    return Some(bone.translation);
                }
            }
        }
        None
    }
}

/// Marker component for temporary sampling entities
#[derive(Component)]
pub struct AnimationSampler {
    pub animation_name: String,
    pub sample_times: Vec<f32>,
    pub current_sample_index: usize,
    pub samples_collected: Vec<(f32, Vec<(String, Vec3, Quat)>)>,
}

/// Extract bone names from an animation clip
fn extract_bone_names_from_clip(clip: &AnimationClip) -> Vec<String> {
    let mut bone_names = Vec::new();

    for (target_id, _curves) in clip.curves() {
        let bone_name = format!("{:?}", target_id);
        if !bone_names.contains(&bone_name) {
            bone_names.push(bone_name);
        }
    }

    bone_names
}

// ============================================================================
// DEBUG: TEST ANIMATION PLAYBACK
// ============================================================================

/// Test system to play parkour animation on character (press 'O' to test)
pub fn test_parkour_animation_playback(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Res<ParkourAnimationLibrary>,
    mut player_query: Query<(&mut AnimationPlayer, &AnimationGraphHandle), With<crate::game::player::Player>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyO) {
        return;
    }

    if !library.loaded {
        warn!("Parkour animations not loaded yet!");
        return;
    }

    let Ok((mut player, current_graph_handle)) = player_query.get_single_mut() else {
        warn!("No player with AnimationPlayer found!");
        return;
    };

    info!("🧪 Testing vault animation playback on character...");

    // Get the current animation graph
    if let Some(graph) = animation_graphs.get_mut(current_graph_handle) {
        // Add the vault animation to the current graph
        let vault_node = graph.add_clip(library.vault_clip.clone(), 1.0, graph.root);

        // Play the vault animation
        player.play(vault_node).repeat();

        info!("✅ Playing vault animation!");
        info!("   If the character animates → Retargeting works! ✅");
        info!("   If nothing happens → Bone names don't match ❌");
        info!("   Press '1' to return to normal walk/run animation");
    } else {
        warn!("Could not access animation graph!");
    }
}

// ============================================================================
// DEBUG: SAMPLE AND PRINT
// ============================================================================

/// Debug system to sample animation on key press
/// Note: This is a simplified version. For full sampling, we would need to:
/// 1. Spawn temporary entity with AnimationPlayer
/// 2. Attach animation graph
/// 3. Play animation and seek to desired time
/// 4. Read bone GlobalTransforms
pub fn debug_sample_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Res<ParkourAnimationLibrary>,
    bone_names: Res<AnimationBoneNames>,
) {
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

    if !library.loaded {
        warn!("Parkour animations not loaded yet!");
        return;
    }

    info!("📊 Parkour animation library ready:");
    info!("   Vault graph: {:?}", library.vault_graph);
    info!("   Vault node: {:?}", library.vault_node);
    info!("   Climb graph: {:?}", library.climb_graph);
    info!("   Climb node: {:?}", library.climb_node);
    info!("");

    // Show bone name verification status
    info!("🦴 Bone name verification:");
    info!("   Character bones collected: {}", bone_names.character_bones.len());
    info!("   Animation bones collected: {}", bone_names.animation_bones.len());
    info!("   Verification complete: {}", bone_names.verified);

    if !bone_names.character_bones.is_empty() {
        info!("   Sample character bones: {:?}", &bone_names.character_bones[..5.min(bone_names.character_bones.len())]);
    }

    if let Some(vault_bones) = bone_names.animation_bones.get("vault") {
        info!("   Vault animation has {} bones", vault_bones.len());
        info!("   Sample vault bones: {:?}", &vault_bones[..5.min(vault_bones.len())]);

        // Manual verification
        if !bone_names.character_bones.is_empty() {
            let mut matched = 0;
            for anim_bone in vault_bones {
                if bone_names.character_bones.contains(anim_bone) {
                    matched += 1;
                }
            }
            let match_percent = (matched as f32 / vault_bones.len() as f32) * 100.0;
            info!("   ✅ Bone matching: {}/{} ({:.1}%)", matched, vault_bones.len(), match_percent);
        }
    }

    info!("");
    info!("💡 To sample animations at runtime:");
    info!("   1. Play animation with AnimationPlayer");
    info!("   2. Use player.seek_to(time) to jump to specific time");
    info!("   3. Read bone GlobalTransform components");
    info!("   4. Store transforms for IK targets");
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourAnimationLibrary>();
    app.init_resource::<AnimationBoneNames>();
    app.init_resource::<SampledParkourPoses>();

    app.add_systems(
        Update,
        (
            check_parkour_animations_loaded,
            collect_character_bone_names,
            collect_animation_bone_names,
            test_parkour_animation_playback,
            debug_sample_animation,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
