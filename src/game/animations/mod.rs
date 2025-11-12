mod animation_assets;
mod animation_controller;
mod controls;
pub mod models;

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

use self::{
    animation_assets::PlayerAnimations,
    animation_controller::{
        apply_animation_state, attach_animation_controller, setup_animation_graph,
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

    // Load animation assets
    app.load_resource::<PlayerAnimations>();

    // Animation systems
    app.add_systems(
        Update,
        (
            setup_animation_graph
                .run_if(resource_added::<PlayerAnimations>)
                .run_if(in_state(Screen::Gameplay)),
            attach_animation_controller.run_if(in_state(Screen::Gameplay)),
            update_animation_state.run_if(in_state(Screen::Gameplay)),
            apply_animation_state.run_if(in_state(Screen::Gameplay)),
            apply_controls.run_if(in_state(Screen::Gameplay)),
        ),
    );
}

