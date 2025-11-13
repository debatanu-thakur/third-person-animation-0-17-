//! Animation Editor screen for creating and editing animation blend configurations.

use bevy::prelude::*;

use crate::{screens::Screen, theme::{palette::*, widget}};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::AnimEditor), spawn_anim_editor);
    app.add_systems(OnExit(Screen::AnimEditor), cleanup_anim_editor);
}

/// Marker component for the animation editor UI
#[derive(Component)]
struct AnimEditorUi;

fn spawn_anim_editor(mut commands: Commands) {
    info!("Entering Animation Editor");

    // Full-screen container
    commands.spawn((
        AnimEditorUi,
        Name::new("AnimEditor Root"),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(NODE_BACKGROUND),
        GlobalZIndex(1),
        children![
            // Top bar with title and back button
            top_bar(),

            // Three-panel layout
            three_panel_layout(),
        ],
    ));
}

fn top_bar() -> impl Bundle {
    (
        Name::new("Top Bar"),
        Node {
            width: percent(100),
            height: px(80),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::bottom(px(2)),
            ..default()
        },
        BorderColor(BUTTON_TEXT),
        children![
            widget::header("Animation Editor"),
            widget::button("Back to Menu", back_to_menu),
        ],
    )
}

fn three_panel_layout() -> impl Bundle {
    (
        Name::new("Three Panel Layout"),
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(10),
            padding: UiRect::all(px(10)),
            ..default()
        },
        children![
            left_panel(),
            center_panel(),
            right_panel(),
        ],
    )
}

fn left_panel() -> impl Bundle {
    (
        Name::new("Left Panel - File Browser"),
        Node {
            width: px(300),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(15)),
            row_gap: px(10),
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderRadius::all(px(8)),
        children![
            widget::label("Files & Configs"),

            // Placeholder content
            widget::label("• GLTF Models"),
            widget::label("• RON Configs"),
            widget::label("• New Config"),
        ],
    )
}

fn center_panel() -> impl Bundle {
    (
        Name::new("Center Panel - 3D Preview"),
        Node {
            flex_grow: 1.0,
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(15)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderRadius::all(px(8)),
        children![
            widget::label("3D Animation Preview"),
            widget::label("(Preview area will be rendered here)"),
        ],
    )
}

fn right_panel() -> impl Bundle {
    (
        Name::new("Right Panel - Controls"),
        Node {
            width: px(400),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(15)),
            row_gap: px(10),
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderRadius::all(px(8)),
        children![
            widget::label("Blend Controls"),

            // Placeholder content
            widget::label("• Speed Slider"),
            widget::label("• Animation Selection"),
            widget::label("• Threshold Controls"),
            widget::label("• Playback Controls"),
        ],
    )
}

fn back_to_menu(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn cleanup_anim_editor(mut commands: Commands, query: Query<Entity, With<AnimEditorUi>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
