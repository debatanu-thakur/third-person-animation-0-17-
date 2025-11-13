//! Animation Editor screen for creating and editing animation blend configurations.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::{screens::Screen, theme::{palette::*, widget}};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>();
    app.add_systems(
        OnEnter(Screen::AnimEditor),
        (scan_asset_files, spawn_anim_editor).chain(),
    );
    app.add_systems(OnExit(Screen::AnimEditor), cleanup_anim_editor);
}

/// Marker component for the animation editor UI
#[derive(Component)]
struct AnimEditorUi;

/// Marker component for the left panel content area
#[derive(Component)]
struct LeftPanelContent;

/// Resource holding the editor state
#[derive(Resource, Default)]
struct EditorState {
    /// List of .glb files found in assets/models
    gltf_files: Vec<PathBuf>,
    /// List of .ron config files found in assets/config
    config_files: Vec<PathBuf>,
    /// Currently selected GLTF file
    selected_gltf: Option<PathBuf>,
    /// Currently selected config file
    selected_config: Option<PathBuf>,
}

/// System to scan the assets folder for GLTF and config files
fn scan_asset_files(mut editor_state: ResMut<EditorState>) {
    use std::fs;

    editor_state.gltf_files.clear();
    editor_state.config_files.clear();

    // Scan for .glb files in assets/models
    if let Ok(entries) = fs::read_dir("assets/models") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("glb") {
                        editor_state.gltf_files.push(path);
                    }
                }
            }
        }
    }

    // Scan for .ron files in assets/config
    if let Ok(entries) = fs::read_dir("assets/config") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                        editor_state.config_files.push(path);
                    }
                }
            }
        }
    }

    // Sort files for consistent display
    editor_state.gltf_files.sort();
    editor_state.config_files.sort();

    info!(
        "Found {} GLTF files and {} config files",
        editor_state.gltf_files.len(),
        editor_state.config_files.len()
    );
}

fn spawn_anim_editor(mut commands: Commands, editor_state: Res<EditorState>) {
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
            three_panel_layout(&editor_state),
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

fn three_panel_layout(editor_state: &EditorState) -> impl Bundle {
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
            left_panel(editor_state),
            center_panel(),
            right_panel(),
        ],
    )
}

fn left_panel(editor_state: &EditorState) -> impl Bundle {
    // Create file list items
    let mut file_items = Vec::new();

    // GLTF Models section
    file_items.push(widget::header("GLTF Models"));
    if editor_state.gltf_files.is_empty() {
        file_items.push(widget::label("No .glb files found"));
    } else {
        for gltf_file in &editor_state.gltf_files {
            if let Some(filename) = gltf_file.file_name().and_then(|f| f.to_str()) {
                file_items.push(file_button(filename, gltf_file.clone()));
            }
        }
    }

    // RON Configs section
    file_items.push(widget::header("Configurations"));
    if editor_state.config_files.is_empty() {
        file_items.push(widget::label("No .ron files found"));
    } else {
        for config_file in &editor_state.config_files {
            if let Some(filename) = config_file.file_name().and_then(|f| f.to_str()) {
                file_items.push(file_button(filename, config_file.clone()));
            }
        }
    }

    // New Config button
    file_items.push(widget::button("+ New Config", create_new_config));

    (
        Name::new("Left Panel - File Browser"),
        Node {
            width: px(300),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(15)),
            row_gap: px(10),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderRadius::all(px(8)),
        children(file_items),
    )
}

/// Marker component for file selection buttons
#[derive(Component, Clone)]
struct FileButton {
    path: PathBuf,
}

/// Create a clickable file button
fn file_button(filename: &str, path: PathBuf) -> impl Bundle {
    let file_path = path.clone();
    (
        FileButton { path },
        Name::new(format!("File: {}", filename)),
        Button,
        Node {
            width: percent(100),
            padding: UiRect::all(px(8)),
            justify_content: JustifyContent::Start,
            ..default()
        },
        BackgroundColor(BUTTON_BACKGROUND),
        BorderRadius::all(px(4)),
        children![(
            Text::new(filename),
            TextFont::from_font_size(18.0),
            TextColor(BUTTON_TEXT),
        )],
    ).observe(move |_trigger: Trigger<Pointer<Click>>| {
        info!("Selected file: {:?}", file_path);
    })
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

fn create_new_config(_: On<Pointer<Click>>) {
    info!("Create new config button clicked");
    // TODO: Implement new config creation
}

fn cleanup_anim_editor(
    mut commands: Commands,
    query: Query<Entity, With<AnimEditorUi>>,
    mut editor_state: ResMut<EditorState>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }

    // Clear editor state
    editor_state.gltf_files.clear();
    editor_state.config_files.clear();
    editor_state.selected_gltf = None;
    editor_state.selected_config = None;
}
