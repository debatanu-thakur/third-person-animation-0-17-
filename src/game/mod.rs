//! The game's menus and transitions between them.

mod animations;
mod camera_controller;
pub mod configs;
mod foot_placement;
mod foot_placement_debug;
mod player;
mod scene;
pub mod target_matching;
pub mod third_person_camera;

use bevy::prelude::*;

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
    ));

    // Configure target matching for Mixamo rigs
    app.insert_resource(target_matching::MaskGroupConfig::for_mixamo());

    // Add diagnostic system for debugging (runs every 3 seconds)
    app.add_systems(
        Update,
        foot_placement_debug::diagnose_foot_placement
            .run_if(on_timer(std::time::Duration::from_secs(3)))
            .run_if(in_state(Screen::Gameplay)),
    );
}
