//! The game's menus and transitions between them.

mod camera_controller;
mod player;
mod scene;
pub mod third_person_camera;
mod animations;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        scene::plugin,
        player::plugin,
        camera_controller::plugin,
        animations::plugin,
    ));
}
