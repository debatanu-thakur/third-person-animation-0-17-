//! The game's menus and transitions between them.

mod camera_controller;
pub mod configs;
mod player;
mod scene;
pub mod third_person_camera;
mod animations;
mod obstacle_detection;
mod parkour_poses;
mod parkour_ik;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        configs::plugin,
        scene::plugin,
        player::plugin,
        camera_controller::plugin,
        animations::plugin,
        obstacle_detection::plugin,
        parkour_poses::plugin,
        parkour_ik::plugin,
    ));
}
