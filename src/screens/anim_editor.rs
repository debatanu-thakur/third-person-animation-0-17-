//! Animation Editor screen for creating and editing animation blend configurations.

use std::{fs, path::PathBuf};

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    gltf::Gltf,
    input::mouse::MouseWheel,
    prelude::*,
    ui::RelativeCursorPosition,
};

use crate::{
    game::configs::assets::{AnimationBlendingConfig, SpeedThresholds},
    screens::Screen,
    theme::{palette::*, widget},
};

// UI Size Constants - reduced to 50% of original sizes for better layout
const FONT_SIZE_HEADER: f32 = 12.0;  // Was 24.0
const FONT_SIZE_NORMAL: f32 = 9.0;   // Was 18.0
const FONT_SIZE_SMALL: f32 = 8.0;    // Was 16.0
const FONT_SIZE_TINY: f32 = 6.0;     // For headers that need to be even smaller
const FONT_SIZE_TITLE: f32 = 16.0;   // Was 32.0

const PADDING_LARGE: f32 = 10.0;     // Was 20.0
const PADDING_MEDIUM: f32 = 7.5;     // Was 15.0
const PADDING_SMALL: f32 = 5.0;      // Was 10.0
const PADDING_TINY: f32 = 4.0;       // Was 8.0
const PADDING_MINI: f32 = 2.0;       // Was 4.0

const GAP_LARGE: f32 = 10.0;         // Was 20.0
const GAP_MEDIUM: f32 = 7.5;         // Was 15.0
const GAP_SMALL: f32 = 5.0;          // Was 10.0
const GAP_TINY: f32 = 4.0;           // Was 8.0

const BUTTON_HEIGHT: f32 = 15.0;     // Was 20.0, reduced further
const BUTTON_FONT_SIZE: f32 = 7.0;   // For button text
const SLIDER_HEIGHT: f32 = 10.0;     // Was 20.0
const TOP_BAR_HEIGHT: f32 = 30.0;    // Was 40.0, reduced further

const PANEL_WIDTH_LEFT: f32 = 150.0; // Was 300.0
const PANEL_WIDTH_RIGHT: f32 = 200.0; // Was 400.0

const BORDER_RADIUS: f32 = 4.0;      // Was 8.0
const BORDER_RADIUS_SMALL: f32 = 2.0; // Was 4.0

// Camera control constants
const CAMERA_ZOOM_SPEED: f32 = 0.2;  // Mouse wheel zoom speed (lower = slower)

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>();
    app.add_message::<FileSelectedEvent>();

    app.add_systems(
        OnEnter(Screen::AnimEditor),
        (scan_asset_files, spawn_anim_editor, setup_preview_scene).chain(),
    );

    app.add_systems(
        Update,
        (
            handle_file_selection,
            load_gltf_animations,
            spawn_preview_character,
            update_preview_animations,
            handle_slider_interaction,
            update_slider_visuals,
            update_slider_labels,
            update_filename_label,
            orbit_camera_controls, // Orbit camera controls
        )
            .run_if(in_state(Screen::AnimEditor)),
    );

    app.add_systems(OnExit(Screen::AnimEditor), cleanup_anim_editor);
}

/// Marker component for the animation editor UI
#[derive(Component)]
struct AnimEditorUi;

/// Marker component for the preview camera
#[derive(Component)]
struct PreviewCamera;

/// Marker component for the spawned preview character
#[derive(Component)]
struct PreviewCharacter;

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

/// Marker component for the filename input label
#[derive(Component)]
struct FilenameLabel;

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
    /// Entity ID of the spawned preview character
    preview_character_entity: Option<Entity>,

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
    /// Filename for saving configuration (without .ron extension)
    config_filename: String,
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
            preview_character_entity: None,
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
            config_filename: String::from("my_blend_config"),
        }
    }
}

/// Message fired when a file is selected
#[derive(Message)]
struct FileSelectedEvent {
    path: PathBuf,
    is_gltf: bool,
}

/// Helper function to recursively scan a directory for files with a specific extension
fn scan_directory_recursive(dir: &str, extension: &str, files: &mut Vec<PathBuf>) {
    use std::fs;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    // Recursively scan subdirectories
                    if let Some(path_str) = path.to_str() {
                        scan_directory_recursive(path_str, extension, files);
                    }
                } else if file_type.is_file() {
                    // Check if file has the desired extension
                    if path.extension().and_then(|s| s.to_str()) == Some(extension) {
                        files.push(path);
                    }
                }
            }
        }
    }
}

/// System to scan the assets folder for GLTF and config files
fn scan_asset_files(mut editor_state: ResMut<EditorState>) {
    use std::fs;

    editor_state.gltf_files.clear();
    editor_state.config_files.clear();

    // Scan for .glb files in assets/models (recursively)
    scan_directory_recursive("assets/models", "glb", &mut editor_state.gltf_files);

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

    // Full-screen container (transparent to show 3D scene)
    let root = commands.spawn((
        AnimEditorUi,
        Name::new("AnimEditor Root"),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        // NO BackgroundColor - let the 3D scene show through!
        GlobalZIndex(1),
    )).id();

    // Spawn children
    commands.entity(root).with_children(|parent| {
        // Top bar
        parent.spawn((
            Name::new("Top Bar"),
            Node {
                width: percent(100),
                height: px(TOP_BAR_HEIGHT),
                padding: UiRect::all(px(PADDING_LARGE)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: UiRect::bottom(px(2)),
                ..default()
            },
            BorderColor::all(BUTTON_TEXT),
        )).with_children(|bar| {
            bar.spawn(widget::header("Animation Editor")); // Keep main title at normal size
            bar.spawn(small_button("Back to Menu", back_to_menu));
        });

        // Three-panel layout
        parent.spawn((
            Name::new("Three Panel Layout"),
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(GAP_SMALL),
                padding: UiRect::all(px(PADDING_SMALL)),
                ..default()
            },
        )).with_children(|panels| {
            // Left Panel - File Browser (inlined from spawn_left_panel)
            panels.spawn((
                Name::new("Left Panel - File Browser"),
                Node {
                    width: px(PANEL_WIDTH_LEFT),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(PADDING_MEDIUM)),
                    row_gap: px(GAP_SMALL),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(PANEL_BACKGROUND),
                BorderRadius::all(px(BORDER_RADIUS)),
            )).with_children(|parent| {
                parent.spawn(small_header("GLTF Models"));

                if editor_state.gltf_files.is_empty() {
                    parent.spawn(widget::label("No .glb files found"));
                } else {
                    for gltf_file in &editor_state.gltf_files {
                        if let Some(filename) = gltf_file.file_name().and_then(|f| f.to_str()) {
                            let file_path = gltf_file.clone();
                            let is_gltf = true;
                            parent
                                .spawn((
                                    Name::new(format!("File: {}", filename)),
                                    Button,
                                    Node {
                                        width: percent(100),
                                        padding: UiRect::all(px(PADDING_TINY)),
                                        justify_content: JustifyContent::Start,
                                        ..default()
                                    },
                                    BackgroundColor(BUTTON_BACKGROUND),
                                    BorderRadius::all(px(BORDER_RADIUS_SMALL)),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(filename),
                                        TextFont::from_font_size(FONT_SIZE_NORMAL),
                                        TextColor(BUTTON_TEXT),
                                    ));
                                })
                                .observe(
                                    move |_trigger: Trigger<Pointer<Click>>,
                                          mut events: MessageWriter<FileSelectedEvent>| {
                                        info!("Selected file: {:?}", file_path);
                                        events.write(FileSelectedEvent {
                                            path: file_path.clone(),
                                            is_gltf,
                                        });
                                    },
                                );
                        }
                    }
                }

                parent.spawn(small_header("Configurations"));

                if editor_state.config_files.is_empty() {
                    parent.spawn(widget::label("No .ron files found"));
                } else {
                    for config_file in &editor_state.config_files {
                        if let Some(filename) = config_file.file_name().and_then(|f| f.to_str()) {
                            let file_path = config_file.clone();
                            let is_gltf = false;
                            parent
                                .spawn((
                                    Name::new(format!("File: {}", filename)),
                                    Button,
                                    Node {
                                        width: percent(100),
                                        padding: UiRect::all(px(PADDING_TINY)),
                                        justify_content: JustifyContent::Start,
                                        ..default()
                                    },
                                    BackgroundColor(BUTTON_BACKGROUND),
                                    BorderRadius::all(px(BORDER_RADIUS_SMALL)),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(filename),
                                        TextFont::from_font_size(FONT_SIZE_NORMAL),
                                        TextColor(BUTTON_TEXT),
                                    ));
                                })
                                .observe(
                                    move |_trigger: Trigger<Pointer<Click>>,
                                          mut events: MessageWriter<FileSelectedEvent>| {
                                        info!("Selected file: {:?}", file_path);
                                        events.write(FileSelectedEvent {
                                            path: file_path.clone(),
                                            is_gltf,
                                        });
                                    },
                                );
                        }
                    }
                }

                parent.spawn(small_button("+ New Config", create_new_config));
            });

            // Center Panel - 3D Preview (completely transparent to show 3D scene)
            panels.spawn((
                Name::new("Center Panel - 3D Preview"),
                Node {
                    flex_grow: 1.0,
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(PADDING_MEDIUM)),
                    ..default()
                },
                // NO BackgroundColor - let the 3D scene show through!
                Pickable::IGNORE, // Don't block picking events
                BorderRadius::all(px(BORDER_RADIUS)),
            )).with_children(|parent| {
                // Info overlay at the top
                parent.spawn((
                    Node {
                        width: percent(100),
                        padding: UiRect::all(px(PADDING_SMALL)),
                        ..default()
                    },
                    BackgroundColor(NODE_BACKGROUND.with_alpha(0.8)),
                    BorderRadius::all(px(BORDER_RADIUS_SMALL)),
                )).with_children(|info| {
                    info.spawn(small_header("3D Preview"));
                });

                // Bottom info
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(PADDING_SMALL),
                        left: px(PADDING_SMALL),
                        padding: UiRect::all(px(PADDING_SMALL)),
                        ..default()
                    },
                    BackgroundColor(NODE_BACKGROUND.with_alpha(0.8)),
                    BorderRadius::all(px(BORDER_RADIUS_SMALL)),
                )).with_children(|info| {
                    info.spawn(widget::label("Load a GLTF file to see the character"));
                });
            });

            // Right Panel - Controls (inlined from spawn_right_panel)
            panels.spawn((
                Name::new("Right Panel - Controls"),
                Node {
                    width: px(PANEL_WIDTH_RIGHT),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(PADDING_MEDIUM)),
                    row_gap: px(GAP_MEDIUM),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(PANEL_BACKGROUND),
                BorderRadius::all(px(BORDER_RADIUS)),
            )).with_children(|parent| {
                // Header
                parent.spawn(small_header("Blend Controls"));

                // Speed preview slider section
                parent.spawn((
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(GAP_TINY),
                        ..default()
                    },
                )).with_children(|section| {
                    section.spawn((
                        Text::new("Preview Speed"),
                        TextFont::from_font_size(FONT_SIZE_HEADER),
                        TextColor(HEADER_TEXT),
                    ));

                    // Inlined slider control for Speed
                    section.spawn((
                        Name::new("Slider: Speed"),
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(GAP_TINY / 2.0),
                            ..default()
                        },
                    )).with_children(|parent| {
                        // Label and value display
                        parent.spawn((
                            Node {
                                width: percent(100),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                        )).with_children(|row| {
                            row.spawn(widget::label("Speed"));
                            row.spawn((
                                Text::new("0.0"),
                                TextFont::from_font_size(FONT_SIZE_NORMAL),
                                TextColor(BUTTON_TEXT),
                                SliderValueLabel(SliderType::Speed),
                            ));
                        });

                        // Slider bar - interactive
                        parent.spawn((
                            Name::new("Slider Bar: Speed"),
                            Slider {
                                slider_type: SliderType::Speed,
                                min: 0.0,
                                max: 10.0,
                            },
                            Button, // Make it clickable
                            RelativeCursorPosition::default(), // Track cursor position for click-based value setting
                            Node {
                                width: percent(100),
                                height: px(SLIDER_HEIGHT),
                                padding: UiRect::all(px(PADDING_MINI)),
                                ..default()
                            },
                            BackgroundColor(NODE_BACKGROUND),
                            BorderRadius::all(px(GAP_SMALL)),
                        )).with_children(|bar| {
                            // Slider handle (filled portion)
                            bar.spawn((
                                Name::new("Slider Handle"),
                                SliderHandle(SliderType::Speed),
                                Node {
                                    width: percent(0), // Will be updated based on value
                                    height: percent(100),
                                    ..default()
                                },
                                BackgroundColor(BUTTON_BACKGROUND),
                                BorderRadius::all(px(8)),
                            ));
                        });
                    });

                    section.spawn(widget::label("Adjust to see animation blending at different speeds"));
                });

                // Divider
                parent.spawn(divider());

                // Animations list section
                parent.spawn((
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(GAP_TINY),
                        ..default()
                    },
                )).with_children(|section| {
                    section.spawn(small_header("Available Animations"));

                    if editor_state.available_animations.is_empty() {
                        section.spawn(widget::label("Load a GLTF file to see animations"));
                    } else {
                        // List all available animations
                        for anim_name in &editor_state.available_animations {
                            section.spawn((
                                Text::new(format!("• {}", anim_name)),
                                TextFont::from_font_size(FONT_SIZE_SMALL),
                                TextColor(LABEL_TEXT),
                            ));
                        }

                        section.spawn((
                            Text::new(format!("Total: {} animations", editor_state.available_animations.len())),
                            TextFont::from_font_size(FONT_SIZE_TINY),
                            TextColor(LABEL_TEXT.with_alpha(0.7)),
                        ));
                    }
                });

                // Divider
                parent.spawn(divider());

                parent.spawn(small_button("⏯ Play/Pause", toggle_playback));
                parent.spawn(small_button("💾 Save", save_configuration));
            });
        });
    });
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

/// Small header variant for AnimEditor
fn small_header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Small Header"),
        Text(text.into()),
        TextFont::from_font_size(FONT_SIZE_TINY),
        TextColor(HEADER_TEXT),
    )
}

/// Small button variant for AnimEditor with custom sizing
fn small_button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text_str = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Small Button Container"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new(format!("Small Button: {}", text_str.clone())),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    Node {
                        width: px(100.0),  // Smaller width
                        height: px(BUTTON_HEIGHT),
                        padding: UiRect::all(px(PADDING_TINY)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderRadius::all(px(BORDER_RADIUS_SMALL)),
                    children![(
                        Name::new("Small Button Text"),
                        Text(text_str),
                        TextFont::from_font_size(BUTTON_FONT_SIZE),
                        TextColor(BUTTON_TEXT),
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}

fn back_to_menu(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn create_new_config(_: On<Pointer<Click>>, mut editor_state: ResMut<EditorState>) {
    info!("Create new config button clicked");

    // Generate a timestamped filename
    use std::time::SystemTime;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("anim_config_{}.ron", timestamp);

    // Create a new configuration with current editor settings
    let config = AnimationBlendingConfig {
        speed_thresholds: SpeedThresholds {
            idle_threshold: editor_state.idle_threshold,
            walk_speed: editor_state.walk_speed,
            run_speed: editor_state.run_speed,
        },
    };

    // Serialize to RON format
    match ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default()) {
        Ok(ron_string) => {
            let path = std::path::PathBuf::from(format!("assets/config/{}", filename));

            // Ensure directory exists
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            // Write to file
            match std::fs::write(&path, ron_string) {
                Ok(_) => {
                    info!("✓ Created new config file: {}", filename);
                    editor_state.config_filename = filename.replace(".ron", "");
                    editor_state.selected_config = Some(path.clone());
                    editor_state.config_files.push(path);
                    editor_state.config_files.sort();
                }
                Err(e) => {
                    error!("Failed to write config file: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to serialize config: {}", e);
        }
    }
}

fn toggle_playback(_: On<Pointer<Click>>, mut editor_state: ResMut<EditorState>) {
    editor_state.is_playing = !editor_state.is_playing;
    info!(
        "Animation playback: {}",
        if editor_state.is_playing {
            "playing"
        } else {
            "paused"
        }
    );
}

fn save_configuration(_: On<Pointer<Click>>, editor_state: Res<EditorState>) {
    info!("Save configuration clicked");

    // Create the configuration structure
    let config = AnimationBlendingConfig {
        speed_thresholds: SpeedThresholds {
            idle_threshold: editor_state.idle_threshold,
            walk_speed: editor_state.walk_speed,
            run_speed: editor_state.run_speed,
        },
    };

    // Serialize to RON format with pretty printing
    let ron_string = match ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default()) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize configuration: {}", e);
            return;
        }
    };

    // Create the filename with .ron extension
    let filename = format!("{}.ron", editor_state.config_filename);
    let filepath = PathBuf::from("assets/config").join(&filename);

    // Ensure the config directory exists
    if let Err(e) = fs::create_dir_all("assets/config") {
        error!("Failed to create config directory: {}", e);
        return;
    }

    // Write the file
    match fs::write(&filepath, ron_string) {
        Ok(_) => {
            info!("✓ Configuration saved to: {:?}", filepath);
            info!("  idle_threshold: {}", editor_state.idle_threshold);
            info!("  walk_speed: {}", editor_state.walk_speed);
            info!("  run_speed: {}", editor_state.run_speed);
        }
        Err(e) => {
            error!("Failed to write configuration file: {}", e);
        }
    }
}

/// System to handle file selection events
fn handle_file_selection(
    mut events: MessageReader<FileSelectedEvent>,
    mut editor_state: ResMut<EditorState>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        if event.is_gltf {
            info!("Loading GLTF file: {:?}", event.path);

            // Convert PathBuf to asset path (remove "assets/" prefix)
            if let Some(asset_path_str) = event.path.to_str() {
                let asset_path_str = asset_path_str.strip_prefix("assets/").unwrap_or(asset_path_str);
                let asset_path = asset_path_str.to_string(); // Own the string

                // Load the GLTF file
                let handle: Handle<Gltf> = asset_server.load(asset_path.clone());

                editor_state.selected_gltf = Some(event.path.clone());
                editor_state.loaded_gltf_handle = Some(handle);
                editor_state.available_animations.clear();
                editor_state.preview_character_entity = None; // Reset to respawn

                info!("GLTF load started for: {}", asset_path);
            }
        } else {
            info!("Loading config file: {:?}", event.path);
            editor_state.selected_config = Some(event.path.clone());

            // Load and parse the RON config file
            match fs::read_to_string(&event.path) {
                Ok(contents) => match ron::de::from_str::<AnimationBlendingConfig>(&contents) {
                    Ok(config) => {
                        // Update editor state with loaded values
                        editor_state.idle_threshold = config.speed_thresholds.idle_threshold;
                        editor_state.walk_speed = config.speed_thresholds.walk_speed;
                        editor_state.run_speed = config.speed_thresholds.run_speed;

                        // Update filename (remove .ron extension and path)
                        if let Some(filename) = event.path.file_stem().and_then(|s| s.to_str()) {
                            editor_state.config_filename = filename.to_string();
                        }

                        info!("✓ Configuration loaded successfully:");
                        info!("  idle_threshold: {}", editor_state.idle_threshold);
                        info!("  walk_speed: {}", editor_state.walk_speed);
                        info!("  run_speed: {}", editor_state.run_speed);
                    }
                    Err(e) => {
                        error!("Failed to parse RON config: {}", e);
                    }
                },
                Err(e) => {
                    error!("Failed to read config file: {}", e);
                }
            }
        }
    }
}

/// System to extract animations from loaded GLTF
fn load_gltf_animations(mut editor_state: ResMut<EditorState>, gltf_assets: Res<Assets<Gltf>>) {
    // Check if we have a GLTF handle and it's loaded
    if let Some(handle) = &editor_state.loaded_gltf_handle {
        if let Some(gltf) = gltf_assets.get(handle) {
            // Only process if we haven't extracted animations yet
            if editor_state.available_animations.is_empty() && !gltf.named_animations.is_empty() {
                // Extract animation names
                let anim_names: Vec<String> =
                    gltf.named_animations.keys().map(|s| s.to_string()).collect();

                info!("Found {} animations: {:?}", anim_names.len(), anim_names);

                editor_state.available_animations = anim_names;
            }
        }
    }
}

/// System to handle slider interactions (click and drag)
fn handle_slider_interaction(
    mut editor_state: ResMut<EditorState>,
    slider_query: Query<(&Slider, &Interaction, &RelativeCursorPosition), Changed<Interaction>>,
) {
    for (slider, interaction, cursor_pos) in &slider_query {
        if *interaction == Interaction::Pressed {
            // Calculate value based on click position
            if let Some(pos) = cursor_pos.normalized {
                // pos.x is 0.0 (left) to 1.0 (right)
                let normalized = pos.x.clamp(0.0, 1.0);
                let new_value = slider.min + (slider.max - slider.min) * normalized;

                set_slider_value(&mut editor_state, slider.slider_type, new_value);
                info!("Slider {:?} updated to: {:.2}", slider.slider_type, new_value);
            }
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
        let normalized =
            ((current_value - slider.min) / (slider.max - slider.min)).clamp(0.0, 1.0);

        // Find and update the handle within this slider
        for child in children.iter() {
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

/// System to setup the 3D preview scene with camera and lighting
fn setup_preview_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera_query: Query<(Entity, &mut Camera, &mut Transform), (With<Camera3d>, Without<PreviewCamera>)>,
) {
    info!("Setting up preview scene");

    // Reuse existing camera and add PreviewCamera component
    if let Ok((camera_entity, mut camera, mut transform)) = camera_query.single_mut() {
        info!("Reusing existing camera for AnimEditor preview");

        // Add PreviewCamera marker component
        commands.entity(camera_entity).insert(PreviewCamera);

        // Configure camera for AnimEditor
        camera.order = -10; // Unique order to avoid ambiguity
        camera.clear_color = ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.12));

        // Position camera for preview
        *transform = Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);

        info!("Camera configured for AnimEditor");
    } else {
        warn!("No suitable camera found to reuse, or camera already has PreviewCamera component");
    }


    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
            affects_lightmapped_meshes: true,
    });

    // Spawn main directional light (key light from above-front)
    commands.spawn((
        AnimEditorUi, // Mark for cleanup
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(2.0, 5.0, 3.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    // Spawn fill light (from the side)
    commands.spawn((
        AnimEditorUi, // Mark for cleanup
        PointLight {
            intensity: 500000.0,
            color: Color::srgb(0.9, 0.9, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 2.0, 2.0),
    ));

    // Spawn back light (rim light)
    commands.spawn((
        AnimEditorUi, // Mark for cleanup
        PointLight {
            intensity: 300000.0,
            color: Color::srgb(1.0, 0.95, 0.9),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, -3.0),
    ));

     // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    info!("Preview scene setup complete with 3-point lighting");
}

/// System to handle orbit camera controls
fn orbit_camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<PreviewCamera>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = camera_query.single_mut() {
        let orbit_point = Vec3::new(0.0, 1.0, 0.0);

        // F key to focus on character (2 units away)
        if keyboard.just_pressed(KeyCode::KeyF) {
            info!("Focusing camera on character");
            // Position camera 2 units in front of character at eye level
            transform.translation = Vec3::new(0.0, 1.0, 2.0);
            transform.look_at(orbit_point, Vec3::Y);
            return; // Skip other controls this frame
        }

        let mut rotation_delta = Vec2::ZERO;
        let mut zoom_delta = 0.0;

        // Keyboard rotation (arrow keys)
        if keyboard.pressed(KeyCode::ArrowLeft) {
            rotation_delta.x += 100.0 * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            rotation_delta.x -= 100.0 * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            rotation_delta.y += 100.0 * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            rotation_delta.y -= 100.0 * time.delta_secs();
        }

        // Mouse wheel zoom (using configurable speed)
        for event in mouse_wheel.read() {
            zoom_delta += event.y * CAMERA_ZOOM_SPEED;
        }

        // Apply rotation
        if rotation_delta != Vec2::ZERO {
            // Horizontal rotation (around Y axis)
            let rotation_y = Quat::from_rotation_y(rotation_delta.x.to_radians());
            let offset = transform.translation - orbit_point;
            transform.translation = orbit_point + rotation_y.mul_vec3(offset);

            // Look at the target
            transform.look_at(orbit_point, Vec3::Y);
        }

        // Apply zoom
        if zoom_delta != 0.0 {
            let direction = (transform.translation - orbit_point).normalize();
            transform.translation -= direction * zoom_delta;

            // Clamp distance
            let min_dist = 1.0;
            let max_dist = 10.0;
            let current_dist = (transform.translation - orbit_point).length();
            if current_dist < min_dist {
                transform.translation = orbit_point + direction * min_dist;
            } else if current_dist > max_dist {
                transform.translation = orbit_point + direction * max_dist;
            }
        }
    }
}

/// System to spawn the preview character when GLTF is loaded
fn spawn_preview_character(
    mut commands: Commands,
    mut editor_state: ResMut<EditorState>,
    gltf_assets: Res<Assets<Gltf>>,
    existing_preview: Query<Entity, With<PreviewCharacter>>,
) {
    // Check if we need to spawn a new character
    if editor_state.preview_character_entity.is_some() {
        return; // Character already spawned
    }

    // Check if we have a loaded GLTF
    if let Some(handle) = &editor_state.loaded_gltf_handle {
        if let Some(gltf) = gltf_assets.get(handle) {
            // Despawn any existing preview character
            for entity in &existing_preview {
                commands.entity(entity).despawn();
            }

            // Get the first scene from the GLTF
            if let Some(scene_handle) = gltf.scenes.first() {
                info!("Spawning preview character");

                // Spawn the character
                let character_entity = commands
                    .spawn((
                        AnimEditorUi, // Mark for cleanup
                        PreviewCharacter,
                        SceneRoot(scene_handle.clone()),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ))
                    .id();

                editor_state.preview_character_entity = Some(character_entity);
                editor_state.is_playing = true; // Auto-play animations
                info!("Preview character spawned: {:?}", character_entity);
                info!("Auto-play enabled");
            }
        }
    }
}

/// System to update preview animations based on current speed and settings
fn update_preview_animations(
    editor_state: Res<EditorState>,
    gltf_assets: Res<Assets<Gltf>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    preview_query: Query<Entity, With<PreviewCharacter>>,
    children_query: Query<&Children>,
) {
    // Only update if state changed or animation is playing
    if !editor_state.is_changed() && !editor_state.is_playing {
        return;
    }

    // Find the animation player in the preview character's children
    for preview_entity in &preview_query {
        if let Some(player_entity) = find_animation_player(preview_entity, &children_query) {
            if let Ok(mut player) = animation_players.get_mut(player_entity) {
                // For now, just play the first available animation
                if let Some(handle) = &editor_state.loaded_gltf_handle {
                    if let Some(gltf) = gltf_assets.get(handle) {
                        if let Some((anim_name, _anim_handle)) =
                            gltf.named_animations.iter().next()
                        {
                            // In Bevy 0.17, we need to get the animation node index from the graph
                            // For now, just play by name if the API supports it
                            // This is a simplified version - full implementation would use animation graph
                            info!("Would play animation: {}", anim_name);

                            // Pause/resume based on is_playing
                            if editor_state.is_playing {
                                player.resume_all();
                            } else {
                                player.pause_all();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper function to recursively find the AnimationPlayer in children
fn find_animation_player(entity: Entity, children_query: &Query<&Children>) -> Option<Entity> {
    // Check if this entity has an AnimationPlayer (we'll check in the query)
    // For now, just return the first child that might have it
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            // Try this child
            return Some(child);
            // In a full implementation, we'd recursively search
        }
    }
    None
}

/// System to update the filename label
fn update_filename_label(
    editor_state: Res<EditorState>,
    mut label_query: Query<&mut Text, With<FilenameLabel>>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for mut text in &mut label_query {
        **text = format!("Filename: {}.ron", editor_state.config_filename);
    }
}

fn cleanup_anim_editor(
    mut commands: Commands,
    query: Query<Entity, With<AnimEditorUi>>,
    camera_query: Query<Entity, With<PreviewCamera>>,
    mut editor_state: ResMut<EditorState>,
) {
    // Remove PreviewCamera component from the camera to restore it
    for camera_entity in &camera_query {
        commands.entity(camera_entity).remove::<PreviewCamera>();
        info!("Removed PreviewCamera component from camera");
    }

    // Despawn all AnimEditor UI elements and lights
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // Clear editor state
    editor_state.gltf_files.clear();
    editor_state.config_files.clear();
    editor_state.selected_gltf = None;
    editor_state.selected_config = None;
    editor_state.loaded_gltf_handle = None;
    editor_state.available_animations.clear();
    editor_state.preview_character_entity = None;
}
