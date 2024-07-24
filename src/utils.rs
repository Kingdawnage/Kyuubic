use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    pbr::wireframe::WireframeConfig,
    prelude::*,
};

#[derive(Resource)]
pub struct WireframeState {
    enabled: bool,
}

impl Default for WireframeState {
    fn default() -> Self {
        Self { enabled: false }
    }
}
#[derive(Resource, Component)]
pub struct FpsText;

#[derive(Component)]
struct FpsRoot;

pub fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[1].value = format!("{value:>4.0}");
            text.sections[1].style.color = if value >= 120.0 {
                Color::srgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                Color::srgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                Color::srgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                Color::srgb(1.0, 0.0, 0.0) // or any other default color
            }
        } else {
            text.sections[1].value = "N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

pub fn setup_fps_counter(mut commands: Commands) {
    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands
        .spawn((
            FpsRoot,
            NodeBundle {
                // give it a dark background for readability
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                // make it "always on top" by setting the Z index to maximum
                // we want it to be displayed over all other UI
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    position_type: PositionType::Absolute,
                    // position it at the top-right corner
                    // 1% away from the top window edge
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    // set bottom/left to Auto, so it can be
                    // automatically sized depending on the text
                    bottom: Val::Auto,
                    left: Val::Auto,
                    // give it some padding for readability
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();
    // create our text
    let text_fps = commands
        .spawn((
            FpsText,
            TextBundle {
                // use two sections, so it is easy to update just the number
                text: Text::from_sections([
                    TextSection {
                        value: "FPS: ".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                    TextSection {
                        value: " N/A".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            // if you want to use your game's font asset,
                            // uncomment this and provide the handle:
                            // font: my_font_handle
                            ..default()
                        },
                    },
                ]),
                ..Default::default()
            },
        ))
        .id();
    commands.entity(root).push_children(&[text_fps]);
}

pub fn toggle_wireframe_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wireframe_state: ResMut<WireframeState>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        if wireframe_config.global {
            wireframe_state.enabled = false;
            wireframe_config.global = false;
            println!("Wireframe disabled");
        } else {
            wireframe_state.enabled = true;
            wireframe_config.global = true;
            println!("Wireframe enabled");
        }
    }
}
