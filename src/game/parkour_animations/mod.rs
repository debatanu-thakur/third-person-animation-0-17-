use bevy::prelude::*;
use std::collections::HashMap;
use crate::{game::animations::animation_controller::AnimationNodes, screens::Screen};
use crate::game::obstacle_detection::detection::{ParkourController, ParkourState};

mod assets;
pub use assets::{ParkourGltfAssets, ParkourAnimations, extract_parkour_animation_clips};

// ============================================================================
// PARKOUR ANIMATION LIBRARY
// ============================================================================

/// Converts ParkourAnimations into ParkourAnimationLibrary
/// This runs once after animations are extracted from GLTF
pub fn create_parkour_library(
    mut commands: Commands,
    parkour_animations: Option<Res<ParkourAnimations>>,
    library: Option<Res<ParkourAnimationLibrary>>,
) {
    // Only run once
    if library.is_some() {
        return;
    }

    let Some(animations) = parkour_animations else {
        return;
    };

    info!("🎨 Creating parkour animation library from extracted animations");

    commands.insert_resource(ParkourAnimationLibrary {
        vault_clip: animations.vault.clone(),
        climb_clip: animations.climb.clone(),
        slide_clip: animations.slide.clone(),
        wall_run_left_clip: animations.wall_run_left.clone(),
        wall_run_right_clip: animations.wall_run_right.clone(),
        roll_clip: animations.roll.clone(),
    });
}

/// Resource holding animation library
#[derive(Resource)]
pub struct ParkourAnimationLibrary {
    pub vault_clip: Handle<AnimationClip>,
    pub climb_clip: Handle<AnimationClip>,
    pub slide_clip: Handle<AnimationClip>,
    pub wall_run_left_clip: Handle<AnimationClip>,
    pub wall_run_right_clip: Handle<AnimationClip>,
    pub roll_clip: Handle<AnimationClip>,
}

// ============================================================================
// ANIMATION SAMPLING DATA STRUCTURES
// ============================================================================

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

// ============================================================================
// DEBUG: TEST ANIMATION PLAYBACK
// ============================================================================

/// Test system to dump bone data to RON files for debugging (press 'O')
pub fn test_parkour_animation_playback(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Option<Res<ParkourAnimationLibrary>>,
    animation_nodes: Option<Res<AnimationNodes>>,
    player_query: Query<(Entity, &AnimationPlayer, &AnimationGraphHandle)>,
    animation_graphs: Res<Assets<AnimationGraph>>,
    animation_clips: Res<Assets<AnimationClip>>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_handle: Res<ParkourGltfAssets>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
) {
    let Some(animation_nodes) = animation_nodes else {
        warn!("Animation nodes not ready!");
        return;
    };
    if !keyboard.just_pressed(KeyCode::KeyO) {
        return;
    }

    let Some(library) = library else {
        warn!("Parkour animations not loaded yet!");
        return;
    };

    info!("🔍 Dumping bone data to RON files...");

    // ========================================
    // 1. DUMP CHARACTER BONE HIERARCHY
    // ========================================
    let mut character_data = String::new();
    character_data.push_str("(\n  character_bones: [\n");

    if let Ok((player_entity, animation_player, graph_handle)) = player_query.single() {
        let anim = if let Some(_) = animation_player.animation(animation_nodes.vault) {true} else {false};
        character_data.push_str(&format!("    // Player Entity: {:?}\n", player_entity));
        character_data.push_str(&format!("    // AnimationPlayer active: {}\n", animation_player.is_playing_animation(animation_nodes.vault)));
        character_data.push_str(&format!("    // AnimationPlayer paused: {}\n", anim));

        // Recursively walk children to find all bones
        fn collect_bones(
            entity: Entity,
            depth: usize,
            children_query: &Query<&Children>,
            name_query: &Query<&Name>,
            output: &mut String,
        ) {
            if let Ok(name) = name_query.get(entity) {
                let indent = "    ".repeat(depth + 1);
                output.push_str(&format!("{}(\n", indent));
                output.push_str(&format!("{}  entity: \"{:?}\",\n", indent, entity));
                output.push_str(&format!("{}  name: \"{}\",\n", indent, name.as_str()));
                output.push_str(&format!("{}  depth: {},\n", indent, depth));
                output.push_str(&format!("{}),\n", indent));
            }

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    collect_bones(child, depth + 1, children_query, name_query, output);
                }
            }
        }

        collect_bones(player_entity, 0, &children_query, &name_query, &mut character_data);
    }

    character_data.push_str("  ],\n");

    // ========================================
    // 2. DUMP ANIMATION GRAPH INFO
    // ========================================
    character_data.push_str("  animation_graph: (\n");

    if let Ok((_, _, graph_handle)) = player_query.single() {
        if let Some(graph) = animation_graphs.get(graph_handle) {
            character_data.push_str(&format!("    root_node: {:?},\n", graph.root));
            character_data.push_str(&format!("    node_count: {},\n", graph.graph.node_count()));


            character_data.push_str("    registered_nodes: (\n");
            character_data.push_str(&format!("      idle: {:?},\n", animation_nodes.idle));
            character_data.push_str(&format!("      walk: {:?},\n", animation_nodes.walk));
            character_data.push_str(&format!("      run: {:?},\n", animation_nodes.run));
            character_data.push_str(&format!("      vault: {:?},\n", animation_nodes.vault));
            character_data.push_str(&format!("      climb: {:?},\n", animation_nodes.climb));
            character_data.push_str(&format!("      slide: {:?},\n", animation_nodes.slide));
            character_data.push_str("    ),\n");

        }
    }

    character_data.push_str("  ),\n");
    character_data.push_str(")\n");

    std::fs::write("assets/bones/character_bones.ron", character_data)
        .expect("Failed to write character_bones.ron");

    // ========================================
    // 3. DUMP VAULT ANIMATION CLIP DATA
    // ========================================
    let mut vault_data = String::new();
    vault_data.push_str("(\n");

    if let Some(vault_clip) = animation_clips.get(&library.vault_clip) {
        vault_data.push_str(&format!("  duration: {},\n", vault_clip.duration()));
        vault_data.push_str("  curves: [\n");

        // Get all curves in the animation
        for (target_id, curves) in vault_clip.curves() {
            vault_data.push_str("    (\n");
            vault_data.push_str(&format!("      target_id: \"{:?}\",\n", target_id));
            vault_data.push_str(&format!("      curve_count: {},\n", curves.len()));

            // Try to get bone name from GLTF named_nodes
            if let Some(gltf) = gltf_assets.get(&gltf_handle.vault_gltf) {
                let mut bone_name = "Unknown".to_string();
                for (name, _node_handle) in gltf.named_nodes.iter() {
                    // We can't easily match AnimationTargetId to node, but we can list names
                    bone_name = format!("Check named_nodes: {:?}", gltf.named_nodes.keys().collect::<Vec<_>>());
                    break;
                }
                vault_data.push_str(&format!("      // GLTF has {} named nodes\n", gltf.named_nodes.len()));
            }

            vault_data.push_str("    ),\n");
        }

        vault_data.push_str("  ],\n");
        vault_data.push_str(&format!("  total_curve_count: {},\n", vault_clip.curves().into_iter().count()));
    }

    // ========================================
    // 4. DUMP VAULT GLTF NAMED NODES
    // ========================================
    vault_data.push_str("  gltf_named_nodes: [\n");

    if let Some(gltf) = gltf_assets.get(&gltf_handle.vault_gltf) {
        for (name, node_handle) in gltf.named_nodes.iter() {
            vault_data.push_str(&format!("    \"{}\",  // Node: {:?}\n", name, node_handle));
        }
        vault_data.push_str(&format!("    // Total: {} named nodes\n", gltf.named_nodes.len()));
    }

    vault_data.push_str("  ],\n");
    vault_data.push_str(")\n");

    std::fs::write("assets/bones/vault_animation_bones.ron", vault_data)
        .expect("Failed to write vault_animation_bones.ron");

    info!("✅ Dumped bone data to:");
    info!("   📄 character_bones.ron - Character bone hierarchy and graph info");
    info!("   📄 vault_animation_bones.ron - Vault animation curves and bone names");
    info!("");
    info!("💡 Push these files to GitHub and I'll analyze them!");
}

// ============================================================================
// DEBUG: SAMPLE AND PRINT
// ============================================================================

/// Debug system to print animation library info (press 'P')
pub fn debug_sample_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Option<Res<ParkourAnimationLibrary>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

    let Some(library) = library else {
        warn!("Parkour animations not loaded yet!");
        return;
    };

    info!("📊 Parkour animation library ready:");
    info!("   Vault clip: {:?}", library.vault_clip);
    info!("   Climb clip: {:?}", library.climb_clip);
    info!("   Slide clip: {:?}", library.slide_clip);
    info!("");
    info!("💡 Press 'V' to test vault animation playback on character");
}

// ============================================================================
// DEBUG: TRIGGER VAULT STATE FOR TESTING
// ============================================================================

/// Test system to trigger vault animation by setting parkour state (press 'V')
pub fn test_trigger_vault_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut ParkourController, With<crate::game::player::Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyV) {
        return;
    }

    let Ok(mut parkour) = player_query.single_mut() else {
        warn!("❌ No player with ParkourController found!");
        return;
    };

    // Toggle between Vaulting and None
    if matches!(parkour.state, ParkourState::Vaulting) {
        parkour.state = ParkourState::Idle;
        info!("🛑 Vault animation stopped (state = None)");
    } else {
        parkour.state = ParkourState::Vaulting;
        info!("");
        info!("🧪 ============================================");
        info!("🧪 VAULT ANIMATION TEST TRIGGERED");
        info!("🧪 ============================================");
        info!("✅ Set parkour state to Vaulting");
        info!("   The animation controller will now play vault animation");
        info!("");
        info!("   👀 WATCH YOUR CHARACTER:");
        info!("   ✅ If character does vaulting motion → RETARGETING WORKS!");
        info!("   ❌ If character freezes/T-poses → Bone mismatch");
        info!("");
        info!("   Press 'V' again to stop");
        info!("🧪 ============================================");
        info!("");
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourGltfAssets>();
    app.init_resource::<SampledParkourPoses>();

    app.add_systems(
        Update,
        (
            // Asset loading (runs once when GLTF loads)
            extract_parkour_animation_clips,
            create_parkour_library,

            // Debug systems
            test_parkour_animation_playback,  // 'O' key - dump bone data
            test_trigger_vault_animation,      // 'V' key - trigger vault animation
            debug_sample_animation,            // 'P' key - print library info
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
