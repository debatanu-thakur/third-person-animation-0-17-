mod controls;
mod models;
use bevy::prelude::*;

use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

use crate::{game::animations::controls::apply_controls, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        TnuaControllerPlugin::new(FixedUpdate),
        TnuaAvian3dPlugin::new(FixedUpdate),
    ));
    app.add_systems(Update,
        apply_controls.run_if(in_state(Screen::Gameplay)));
}


