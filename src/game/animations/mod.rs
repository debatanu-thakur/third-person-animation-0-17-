mod animation_assets;
mod animation_controller;
mod controls;
pub mod models;

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

use self::{
    animation_assets::{PlayerAnimationGltfs, PlayerAnimations},
    animation_controller::{
        apply_animation_state, attach_animation_controllers, extract_animations_from_gltf,
        setup_animation_graph, update_animation_state,
    },
    controls::apply_controls,
};

pub(super) fn plugin(app: &mut App) {
    // Tnua controller plugins
    app.add_plugins((
        TnuaControllerPlugin::new(FixedUpdate),
        TnuaAvian3dPlugin::new(FixedUpdate),
    ));

    // Load GLTF files containing animations
    app.load_resource::<PlayerAnimationGltfs>();

    // Animation systems - multi-stage loading:
    // 1. Load GLTF files (PlayerAnimationGltfs)
    // 2. Extract animation clips from GLTFs -> PlayerAnimations resource
    // 3. Build animation graph from clips -> AnimationNodes resource
    // 4. Apply animations to player
    app.add_systems(
        Update,
        (
            // Stage 1: Extract animations from loaded GLTF files
            extract_animations_from_gltf
                .run_if(resource_added::<PlayerAnimationGltfs>)
                .run_if(in_state(Screen::Gameplay))
                .run_if(not(resource_exists::<PlayerAnimations>)),
            // Stage 2: Setup animation graph once clips are extracted
            setup_animation_graph
                .run_if(resource_added::<PlayerAnimations>)
                .run_if(in_state(Screen::Gameplay))
                .run_if(not(resource_exists::<animation_controller::AnimationNodes>)),
            // Stage 3: Attach and update animations
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
    // app.add_systems(Update, extract_animations_from_gltf.run_if(in_state(Screen::Gameplay)));
    // app.add_systems(Update, setup_animation_graph.run_if(in_state(Screen::Gameplay)));
    // app.add_systems(Update, attach_animation_controller.run_if(in_state(Screen::Gameplay)));
    // app.add_systems(Update, update_animation_state.run_if(in_state(Screen::Gameplay)));
    // app.add_systems(Update, apply_animation_state.run_if(in_state(Screen::Gameplay)));
    // app.add_systems(Update, apply_controls.run_if(in_state(Screen::Gameplay)));
}

