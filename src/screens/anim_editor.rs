//! Animation Editor screen for creating and editing animation blend configurations.

use std::{fs, path::PathBuf};

use bevy::{gltf::Gltf, prelude::*};

use crate::{
    game::configs::assets::{AnimationBlendingConfig, SpeedThresholds},
    screens::Screen,
    theme::{palette::*, widget},
};

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
        BackgroundColor(NODE_BACKGROUND),
        GlobalZIndex(1),
    )).id();

    // Spawn children
    commands.entity(root).with_children(|parent| {
        // Top bar
        parent.spawn((
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
            BorderColor::all(BUTTON_TEXT),
        )).with_children(|bar| {
            bar.spawn(widget::header("Animation Editor"));
            bar.spawn(widget::button("Back to Menu", back_to_menu));
        });

        // Three-panel layout
        parent.spawn((
            Name::new("Three Panel Layout"),
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(10),
                padding: UiRect::all(px(10)),
                ..default()
            },
        )).with_children(|panels| {
            // Left Panel - File Browser (inlined from spawn_left_panel)
            panels.spawn((
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
            )).with_children(|parent| {
                parent.spawn(widget::header("GLTF Models"));

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
                                        padding: UiRect::all(px(8)),
                                        justify_content: JustifyContent::Start,
                                        ..default()
                                    },
                                    BackgroundColor(BUTTON_BACKGROUND),
                                    BorderRadius::all(px(4)),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(filename),
                                        TextFont::from_font_size(18.0),
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

                parent.spawn(widget::header("Configurations"));

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
                                        padding: UiRect::all(px(8)),
                                        justify_content: JustifyContent::Start,
                                        ..default()
                                    },
                                    BackgroundColor(BUTTON_BACKGROUND),
                                    BorderRadius::all(px(4)),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(filename),
                                        TextFont::from_font_size(18.0),
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

                parent.spawn(widget::button("+ New Config", create_new_config));
            });

            // Center Panel - 3D Preview (inlined from spawn_center_panel)
            panels.spawn((
                Name::new("Center Panel - 3D Preview"),
                Node {
                    flex_grow: 1.0,
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(15)),
                    ..default()
                },
                BackgroundColor(PANEL_BACKGROUND.with_alpha(0.3)), // Semi-transparent to see 3D scene
                BorderRadius::all(px(8)),
            )).with_children(|parent| {
                // Info overlay at the top
                parent.spawn((
                    Node {
                        width: percent(100),
                        padding: UiRect::all(px(10)),
                        ..default()
                    },
                    BackgroundColor(NODE_BACKGROUND.with_alpha(0.8)),
                    BorderRadius::all(px(4)),
                )).with_children(|info| {
                    info.spawn(widget::header("3D Preview"));
                });

                // Bottom info
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(10),
                        left: px(10),
                        padding: UiRect::all(px(10)),
                        ..default()
                    },
                    BackgroundColor(NODE_BACKGROUND.with_alpha(0.8)),
                    BorderRadius::all(px(4)),
                )).with_children(|info| {
                    info.spawn(widget::label("Load a GLTF file to see the character"));
                });
            });

            // Right Panel - Controls (inlined from spawn_right_panel)
            panels.spawn((
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
            )).with_children(|parent| {
                // Header
                parent.spawn(widget::header("Blend Controls"));

                // Speed preview slider section
                parent.spawn((
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        ..default()
                    },
                )).with_children(|section| {
                    section.spawn((
                        Text::new("Preview Speed"),
                        TextFont::from_font_size(20.0),
                        TextColor(HEADER_TEXT),
                    ));

                    // Inlined slider control for Speed
                    section.spawn((
                        Name::new("Slider: Speed"),
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(5),
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
                                TextFont::from_font_size(18.0),
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
                            Node {
                                width: percent(100),
                                height: px(20),
                                padding: UiRect::all(px(2)),
                                ..default()
                            },
                            BackgroundColor(NODE_BACKGROUND),
                            BorderRadius::all(px(10)),
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

                parent.spawn(widget::button("⏯ Play/Pause", toggle_playback));
                parent.spawn(widget::button("💾 Save Configuration", save_configuration));
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

fn back_to_menu(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn create_new_config(_: On<Pointer<Click>>) {
    info!("Create new config button clicked");
    // TODO: Implement new config creation
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
    slider_query: Query<(&Slider, &Interaction), Changed<Interaction>>,
) {
    for (slider, interaction) in &slider_query {
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
fn setup_preview_scene(mut commands: Commands) {
    info!("Setting up preview scene");

    // Spawn directional light
    commands.spawn((
        AnimEditorUi, // Mark for cleanup
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn preview camera
    commands.spawn((
        AnimEditorUi, // Mark for cleanup
        PreviewCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 3.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        Camera {
            // Render to the main surface
            order: -1, // Render before UI
            ..default()
        },
    ));

    info!("Preview scene setup complete");
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
                info!("Preview character spawned: {:?}", character_entity);
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
    mut editor_state: ResMut<EditorState>,
) {
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
