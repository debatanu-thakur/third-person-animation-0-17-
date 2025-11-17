//! The game's menus and transitions between them.

mod camera_controller;
pub mod configs;
mod player;
mod scene;
pub mod target_matching;
pub mod third_person_camera;
mod animations;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        configs::plugin,
        scene::plugin,
        player::plugin,
        camera_controller::plugin,
        animations::plugin,
        target_matching::TargetMatchingPlugin,
    ));

    // Configure target matching for Mixamo rigs
    app.insert_resource(target_matching::MaskGroupConfig::for_mixamo());
}
