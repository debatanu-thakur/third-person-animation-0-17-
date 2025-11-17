//! The game's menus and transitions between them.

mod animations;
mod camera_controller;
pub mod configs;
mod foot_placement;
mod foot_placement_debug;
mod hand_placement;
mod player;
mod scene;
pub mod target_matching;
mod target_matching_debug;
pub mod third_person_camera;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        configs::plugin,
        scene::plugin,
        player::plugin,
        camera_controller::plugin,
        animations::plugin,
        target_matching::TargetMatchingPlugin,
        foot_placement::FootPlacementPlugin,
        hand_placement::HandPlacementPlugin,
    ));

    // Configure target matching for Mixamo rigs
    app.insert_resource(target_matching::MaskGroupConfig::for_mixamo());

    // Add diagnostic systems for debugging
    app.add_systems(
        Update,
        (
            foot_placement_debug::diagnose_foot_placement
                .run_if(on_timer(std::time::Duration::from_secs(3))),
            target_matching_debug::diagnose_bone_components
                .run_if(on_timer(std::time::Duration::from_secs(5))),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}
