//! Development tools for the game. This plugin is only enabled in dev builds.

use avian3d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::{input_just_pressed, input_toggle_active}, prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, (log_transitions::<Screen>, toggle_physics_debug));
    app.add_plugins((
        EguiPlugin::default(),
        PhysicsDebugPlugin::default(),
    ));
    app.add_plugins(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)));
    // TODO: Re-enable when bevy_ui_debug feature is restored
    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
fn toggle_physics_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut store: ResMut<GizmoConfigStore>,
) {
    if keys.just_pressed(KeyCode::F3) {
        let (config, _) = store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
        info!("Physics debug rendering: {}", if config.enabled { "ON" } else { "OFF" });
    }
}
