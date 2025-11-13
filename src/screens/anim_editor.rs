//! Animation Editor screen for creating and editing animation blend configurations.

use std::path::PathBuf;

use bevy::{gltf::Gltf, prelude::*};

use crate::{screens::Screen, theme::{palette::*, widget}};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>();
    app.add_event::<FileSelectedEvent>();

    app.add_systems(
        OnEnter(Screen::AnimEditor),
        (scan_asset_files, spawn_anim_editor).chain(),
    );

    app.add_systems(
        Update,
        (
            handle_file_selection,
            load_gltf_animations,
            handle_slider_interaction,
            update_slider_visuals,
            update_slider_labels,
        ).run_if(in_state(Screen::AnimEditor)),
    );

    app.add_systems(OnExit(Screen::AnimEditor), cleanup_anim_editor);
}

/// Marker component for the animation editor UI
#[derive(Component)]
struct AnimEditorUi;

/// Marker component for the left panel content area
#[derive(Component)]
struct LeftPanelContent;

/// Marker component for sliders
#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum SliderType {
    Speed,
    IdleThreshold,
    WalkSpeed,
    RunSpeed,
    PlaybackSpeed,
}

/// Component for slider configuration
#[derive(Component)]
struct Slider {
    slider_type: SliderType,
    min: f32,
    max: f32,
}

/// Marker component for the slider handle (filled portion)
#[derive(Component)]
struct SliderHandle(SliderType);

/// Marker component for slider value labels
#[derive(Component)]
struct SliderValueLabel(SliderType);

/// Marker component for animation selection buttons
#[derive(Component)]
enum AnimationType {
    Idle,
    Walk,
    Run,
    Jump,
}

/// Resource holding the editor state
#[derive(Resource)]
struct EditorState {
    /// List of .glb files found in assets/models
    gltf_files: Vec<PathBuf>,
    /// List of .ron config files found in assets/config
    config_files: Vec<PathBuf>,
    /// Currently selected GLTF file path
    selected_gltf: Option<PathBuf>,
    /// Currently selected config file path
    selected_config: Option<PathBuf>,
    /// Handle to the loaded GLTF asset
    loaded_gltf_handle: Option<Handle<Gltf>>,
    /// List of animation names extracted from the loaded GLTF
    available_animations: Vec<String>,

    // Configuration being edited
    /// Current speed slider value (for preview)
    current_speed: f32,
    /// Idle threshold configuration
    idle_threshold: f32,
    /// Walk speed threshold
    walk_speed: f32,
    /// Run speed threshold
    run_speed: f32,
    /// Selected idle animation
    selected_idle_anim: Option<String>,
    /// Selected walk animation
    selected_walk_anim: Option<String>,
    /// Selected run animation
    selected_run_anim: Option<String>,
    /// Selected jump animation
    selected_jump_anim: Option<String>,
    /// Playback speed multiplier
    playback_speed: f32,
    /// Is animation playing
    is_playing: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            gltf_files: Vec::new(),
            config_files: Vec::new(),
            selected_gltf: None,
            selected_config: None,
            loaded_gltf_handle: None,
            available_animations: Vec::new(),
            current_speed: 0.0,
            idle_threshold: 0.1,
            walk_speed: 2.0,
            run_speed: 8.0,
            selected_idle_anim: None,
            selected_walk_anim: None,
            selected_run_anim: None,
            selected_jump_anim: None,
            playback_speed: 1.0,
            is_playing: true,
        }
    }
}

/// Event fired when a file is selected
#[derive(Event)]
struct FileSelectedEvent {
    path: PathBuf,
    is_gltf: bool,
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
    let is_gltf = path.extension().and_then(|s| s.to_str()) == Some("glb");

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
    ).observe(move |_trigger: Trigger<Pointer<Click>>, mut events: EventWriter<FileSelectedEvent>| {
        info!("Selected file: {:?}", file_path);
        events.send(FileSelectedEvent {
            path: file_path.clone(),
            is_gltf,
        });
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
            row_gap: px(15),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderRadius::all(px(8)),
        children![
            // Header
            widget::header("Blend Controls"),

            // Speed preview slider
            control_section("Preview Speed", vec![
                slider_control("Speed", SliderType::Speed, 0.0, 10.0),
                widget::label("Adjust to see animation blending at different speeds"),
            ]),

            // Divider
            divider(),

            // Animation selection
            control_section("Animation Assignment", vec![
                widget::label("Idle:"),
                animation_selector_placeholder(AnimationType::Idle),
                widget::label("Walk:"),
                animation_selector_placeholder(AnimationType::Walk),
                widget::label("Run:"),
                animation_selector_placeholder(AnimationType::Run),
                widget::label("Jump:"),
                animation_selector_placeholder(AnimationType::Jump),
            ]),

            // Divider
            divider(),

            // Threshold controls
            control_section("Speed Thresholds", vec![
                slider_control("Idle Threshold", SliderType::IdleThreshold, 0.0, 1.0),
                slider_control("Walk Speed", SliderType::WalkSpeed, 0.0, 5.0),
                slider_control("Run Speed", SliderType::RunSpeed, 2.0, 15.0),
            ]),

            // Divider
            divider(),

            // Playback controls
            control_section("Playback", vec![
                widget::button("⏯ Play/Pause", toggle_playback),
                slider_control("Speed Multiplier", SliderType::PlaybackSpeed, 0.1, 3.0),
            ]),

            // Divider
            divider(),

            // Save button
            widget::button("💾 Save Configuration", save_configuration),
        ],
    )
}

/// Create a labeled control section
fn control_section(title: &str, controls: Vec<impl Bundle>) -> impl Bundle {
    (
        Name::new(format!("Section: {}", title)),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        children![
            (
                Text::new(title),
                TextFont::from_font_size(20.0).with_font(default()),
                TextColor(HEADER_TEXT),
            ),
        ].into_iter().chain(controls.into_iter()).collect::<Vec<_>>(),
    )
}

/// Create a visual divider
fn divider() -> impl Bundle {
    (
        Name::new("Divider"),
        Node {
            width: percent(100),
            height: px(1),
            ..default()
        },
        BackgroundColor(BUTTON_TEXT.with_alpha(0.3)),
    )
}

/// Create a slider control with label
fn slider_control(label: &str, slider_type: SliderType, min: f32, max: f32) -> impl Bundle {
    (
        Name::new(format!("Slider: {}", label)),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
        },
        children![
            // Label and value display
            (
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    widget::label(label),
                    (
                        Text::new("0.0"),
                        TextFont::from_font_size(18.0),
                        TextColor(BUTTON_TEXT),
                        SliderValueLabel(slider_type),
                    ),
                ],
            ),
            // Slider bar - interactive
            (
                Name::new(format!("Slider Bar: {}", label)),
                Slider {
                    slider_type,
                    min,
                    max,
                },
                Button, // Make it clickable
                Node {
                    width: percent(100),
                    height: px(20),
                    padding: UiRect::all(px(2)),
                    ..default()
                },
                BackgroundColor(NODE_BACKGROUND),
                BorderRadius::all(px(10)),
                children![
                    // Slider handle (filled portion)
                    (
                        Name::new("Slider Handle"),
                        SliderHandle(slider_type),
                        Node {
                            width: percent(0), // Will be updated based on value
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(BUTTON_BACKGROUND),
                        BorderRadius::all(px(8)),
                    ),
                ],
            ),
        ],
    )
}

/// Placeholder for animation selector (will be replaced with dropdown)
fn animation_selector_placeholder(anim_type: AnimationType) -> impl Bundle {
    (
        Node {
            width: percent(100),
            padding: UiRect::all(px(8)),
            margin: UiRect::bottom(px(5)),
            ..default()
        },
        BackgroundColor(NODE_BACKGROUND),
        BorderRadius::all(px(4)),
        children![
            widget::label("Click GLTF file to load animations"),
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

fn toggle_playback(_: On<Pointer<Click>>, mut editor_state: ResMut<EditorState>) {
    editor_state.is_playing = !editor_state.is_playing;
    info!("Animation playback: {}", if editor_state.is_playing { "playing" } else { "paused" });
}

fn save_configuration(_: On<Pointer<Click>>, editor_state: Res<EditorState>) {
    info!("Save configuration clicked");
    info!("Current config: idle_threshold={}, walk_speed={}, run_speed={}",
        editor_state.idle_threshold,
        editor_state.walk_speed,
        editor_state.run_speed
    );
    // TODO: Implement actual file saving
}

/// System to handle file selection events
fn handle_file_selection(
    mut events: EventReader<FileSelectedEvent>,
    mut editor_state: ResMut<EditorState>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        if event.is_gltf {
            info!("Loading GLTF file: {:?}", event.path);

            // Convert PathBuf to asset path (remove "assets/" prefix)
            if let Some(asset_path) = event.path.to_str() {
                let asset_path = asset_path.strip_prefix("assets/").unwrap_or(asset_path);

                // Load the GLTF file
                let handle: Handle<Gltf> = asset_server.load(asset_path);

                editor_state.selected_gltf = Some(event.path.clone());
                editor_state.loaded_gltf_handle = Some(handle);
                editor_state.available_animations.clear();

                info!("GLTF load started for: {}", asset_path);
            }
        } else {
            info!("Loading config file: {:?}", event.path);
            editor_state.selected_config = Some(event.path.clone());
            // TODO: Load and parse the RON config file
        }
    }
}

/// System to extract animations from loaded GLTF
fn load_gltf_animations(
    mut editor_state: ResMut<EditorState>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    // Check if we have a GLTF handle and it's loaded
    if let Some(handle) = &editor_state.loaded_gltf_handle {
        if let Some(gltf) = gltf_assets.get(handle) {
            // Only process if we haven't extracted animations yet
            if editor_state.available_animations.is_empty() && !gltf.named_animations.is_empty() {
                // Extract animation names
                let anim_names: Vec<String> = gltf.named_animations.keys()
                    .map(|s| s.to_string())
                    .collect();

                info!("Found {} animations: {:?}", anim_names.len(), anim_names);

                editor_state.available_animations = anim_names;
            }
        }
    }
}

/// System to handle slider interactions (click and drag)
fn handle_slider_interaction(
    mut editor_state: ResMut<EditorState>,
    slider_query: Query<(&Slider, &Node, &GlobalTransform, &Interaction), Changed<Interaction>>,
) {
    for (slider, node, transform, interaction) in &slider_query {
        if *interaction == Interaction::Pressed {
            // TODO: For now, just increment the value on click
            // In a full implementation, we'd calculate based on mouse position
            let current_value = get_slider_value(&editor_state, slider.slider_type);
            let range = slider.max - slider.min;
            let new_value = (current_value + range * 0.1).min(slider.max);

            set_slider_value(&mut editor_state, slider.slider_type, new_value);
            info!("Slider {:?} updated to: {:.2}", slider.slider_type, new_value);
        }
    }
}

/// System to update slider visual representation based on current values
fn update_slider_visuals(
    editor_state: Res<EditorState>,
    mut handle_query: Query<(&SliderHandle, &mut Node)>,
    slider_query: Query<(&Slider, &Children)>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for (slider, children) in &slider_query {
        let current_value = get_slider_value(&editor_state, slider.slider_type);
        let normalized = ((current_value - slider.min) / (slider.max - slider.min)).clamp(0.0, 1.0);

        // Find and update the handle within this slider
        for &child in children.iter() {
            if let Ok((handle, mut node)) = handle_query.get_mut(child) {
                if handle.0 == slider.slider_type {
                    node.width = Val::Percent(normalized * 100.0);
                }
            }
        }
    }
}

/// System to update slider value labels
fn update_slider_labels(
    editor_state: Res<EditorState>,
    mut label_query: Query<(&SliderValueLabel, &mut Text)>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for (label, mut text) in &mut label_query {
        let value = get_slider_value(&editor_state, label.0);
        **text = format!("{:.2}", value);
    }
}

/// Helper function to get slider value from EditorState
fn get_slider_value(state: &EditorState, slider_type: SliderType) -> f32 {
    match slider_type {
        SliderType::Speed => state.current_speed,
        SliderType::IdleThreshold => state.idle_threshold,
        SliderType::WalkSpeed => state.walk_speed,
        SliderType::RunSpeed => state.run_speed,
        SliderType::PlaybackSpeed => state.playback_speed,
    }
}

/// Helper function to set slider value in EditorState
fn set_slider_value(state: &mut EditorState, slider_type: SliderType, value: f32) {
    match slider_type {
        SliderType::Speed => state.current_speed = value,
        SliderType::IdleThreshold => state.idle_threshold = value,
        SliderType::WalkSpeed => state.walk_speed = value,
        SliderType::RunSpeed => state.run_speed = value,
        SliderType::PlaybackSpeed => state.playback_speed = value,
    }
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
    editor_state.loaded_gltf_handle = None;
    editor_state.available_animations.clear();
}
