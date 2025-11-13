mod animation_controller;
mod controls;
pub mod models;

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::screens::Screen;

use self::{
    animation_controller::{
        apply_animation_state, attach_animation_controllers, setup_animation_graph,
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

    // Animation systems - multi-stage loading:
    // 1. PlayerGltfAsset is loaded (handled in player module)
    // 2. PlayerAssets is extracted from GLTF (handled in player module)
    // 3. Build animation graph from PlayerAssets -> AnimationNodes resource
    // 4. Apply animations to player
    app.add_systems(
        Update,
        (
            // Setup animation graph once PlayerAssets is available
            setup_animation_graph
                .run_if(resource_added::<crate::game::player::PlayerAssets>)
                .run_if(in_state(Screen::Gameplay))
                .run_if(not(resource_exists::<animation_controller::AnimationNodes>)),
            // Attach and update animations
            attach_animation_controllers
                .run_if(in_state(Screen::Gameplay))
                .run_if(resource_exists::<animation_controller::AnimationNodes>),
            update_animation_state
                .run_if(in_state(Screen::Gameplay))
                .run_if(resource_exists::<animation_controller::AnimationNodes>),
            apply_animation_state
                .run_if(in_state(Screen::Gameplay))
                .run_if(resource_exists::<animation_controller::AnimationNodes>),
            apply_controls.run_if(in_state(Screen::Gameplay)),
        ),
    );
}

