use bevy::prelude::*;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    time::Time,
};

use crate::Chalk;

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup).add_system(update);
    }
}

#[derive(Component)]
struct DebugText;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

fn update(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut text: Query<&mut Text, With<DebugText>>,
    mut pen: Query<&mut Chalk>,
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
    let pen = format!("{x:+05}:{y:+05} {pressed}");

    text.sections[0].value = format!("{fps}\n{frame_time}\n{pen}\n",);
}
