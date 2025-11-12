mod animation_scene;

use bevy::prelude::*;

use crate::{
    game::{player::SpawnPlayer, scene::animation_scene::spawn_animation_test_scene},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_animation_test_scene, spawn_level).chain(),
    );
}

pub fn spawn_level(world: &mut World) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    SpawnPlayer {
        position: Vec3::new(0., 5., 0.),
    }
    .apply(world);
}
