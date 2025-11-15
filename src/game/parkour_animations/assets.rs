use bevy::{gltf::Gltf, prelude::*};

/// Resource holding the vault animation GLTF
#[derive(Resource, Asset, Reflect, Clone)]
pub struct VaultGltfAsset {
    /// The vault animation GLB file
    #[dependency]
    pub gltf: Handle<Gltf>,
}

impl VaultGltfAsset {
    /// Path to the vault animation GLB file
    pub const PATH: &'static str = "models/animations/vault.glb";
}

impl FromWorld for VaultGltfAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            gltf: assets.load(Self::PATH),
        }
    }
}

/// Resource holding extracted vault animation bone information
/// Created once the GLTF is fully loaded
#[derive(Resource, Clone)]
pub struct VaultAnimationInfo {
    /// List of bone names found in the animation
    pub bone_names: Vec<String>,
    /// Whether the animation uses mixamorig: prefix
    pub has_mixamorig: bool,
    /// Whether the animation uses mixamorig12: prefix
    pub has_mixamorig12: bool,
}

/// Extracts bone names from the loaded vault animation GLTF
/// This system runs once the VaultGltfAsset is loaded
pub fn extract_vault_bone_info(
    mut commands: Commands,
    vault_asset: Res<VaultGltfAsset>,
    gltf_assets: Res<Assets<Gltf>>,
    vault_info: Option<Res<VaultAnimationInfo>>,
) {
    // Only run once - if VaultAnimationInfo already exists, we're done
    if vault_info.is_some() {
        return;
    }

    // Try to get the loaded GLTF
    let Some(gltf) = gltf_assets.get(&vault_asset.gltf) else {
        // GLTF not loaded yet, try again next frame
        return;
    };

    // Extract bone names from named nodes
    let mut bone_names: Vec<String> = gltf.named_nodes.keys().cloned().map(|k| k.as_ref().to_string()).collect();
    bone_names.sort();

    // Check for common prefixes
    let has_mixamorig = bone_names.iter().any(|n| n.starts_with("mixamorig:"));
    let has_mixamorig12 = bone_names.iter().any(|n| n.starts_with("mixamorig12:"));

    info!("✅ Vault animation GLTF loaded!");
    info!("   Total bones/nodes: {}", bone_names.len());
    info!("   Has 'mixamorig:' prefix: {}", has_mixamorig);
    info!("   Has 'mixamorig12:' prefix: {}", has_mixamorig12);

    if bone_names.len() > 0 {
        info!("   First 10 bones:");
        for (i, bone_name) in bone_names.iter().take(10).enumerate() {
            info!("     {}: {}", i + 1, bone_name);
        }
    }

    // Create resource
    commands.insert_resource(VaultAnimationInfo {
        bone_names,
        has_mixamorig,
        has_mixamorig12,
    });
}
