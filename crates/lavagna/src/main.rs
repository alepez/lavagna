use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    input::{mouse::*, ButtonState},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (640., 480.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        .add_system(mouse_system)
        .add_system(debug_test_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((Pen::new(), LocalPen));

    commands.spawn((
        TextBundle::from_section(
            "--",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 20.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        DebugText,
    ));
}

fn mouse_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut pen: Query<&mut Pen, With<LocalPen>>,
) {
    let mut pen = pen.single_mut();
    for event in mouse_button_input_events.iter() {
        match event {
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            } => {
                pen.pressed = true;
            }
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
            } => {
                pen.pressed = false;
            }
            _ => {}
        }
    }

    for event in cursor_moved_events.iter() {
        pen.x = event.position[0] as i64;
        pen.y = event.position[1] as i64;
    }
}

#[derive(Component)]
struct Pen {
    pressed: bool,
    x: i64,
    y: i64,
}

#[derive(Component)]
struct LocalPen;

#[derive(Component)]
struct DebugText;

impl Pen {
    fn new() -> Self {
        Self {
            pressed: false,
            x: 0,
            y: 0,
        }
    }
}

fn debug_test_system(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut text: Query<&mut Text, With<DebugText>>,
    mut pen: Query<&mut Pen>,
) {
    let mut text = text.single_mut();
    let pen = pen.single_mut();

    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|x| x.smoothed())
        .map(|x| format!("{:.1} fps", x))
        .unwrap_or("".to_owned());

    let frame_time = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|x| x.smoothed())
        .unwrap_or_else(|| time.delta_seconds_f64());
    let frame_time = format!("{:.3} ms/frame", frame_time);

    let x = pen.x;
    let y = pen.y;
    let pressed = pen.pressed;
    let pen = format!("{x}:{y} {pressed}");

    text.sections[0].value = format!("{fps}\n{frame_time}\n{pen}\n",);
}
