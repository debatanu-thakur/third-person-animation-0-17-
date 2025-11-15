// mod blending;  // Old blending system - not currently used
pub mod animation_controller;
mod controls;
pub mod models;

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::screens::Screen;

use self::{
    // blending::{
    //     setup_animation_graph,
    //     update_animation_state,
    //     PreviousAnimationState,
    // },
    animation_controller::{
        setup_animation_graph,
        update_animation_state,
    },
    controls::apply_controls,
};

pub(super) fn plugin(app: &mut App) {
    // Tnua controller plugins
    app.add_plugins((
        TnuaControllerPlugin::new(FixedUpdate),
        TnuaAvian3dPlugin::new(FixedUpdate),
    ));

    // Initialize animation state tracking
    // app.init_resource::<PreviousAnimationState>();

    // Animation systems - multi-stage loading:
    // 1. PlayerGltfAsset is loaded (handled in player module)
    // 2. PlayerAssets is extracted from GLTF (handled in player module)
    // 3. Build animation graph from PlayerAssets -> AnimationNodes resource
    // 4. Apply animations to player
    app.add_systems(
        FixedUpdate,
        (
            // Setup animation graph once PlayerAssets is available
            setup_animation_graph,
            // Attach and update animations
            update_animation_state,

            apply_controls.in_set(TnuaUserControlsSystems),
        ).run_if(in_state(Screen::Gameplay)),
    );
}

