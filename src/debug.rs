#![allow(clippy::needless_pass_by_value)]

use bevy::prelude::*;

use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    time::Time,
};

use crate::local_chalk::LocalChalk;
use crate::Stats;

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, update);
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
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        DebugText,
    ));
}

fn update(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut text: Query<&mut Text, With<DebugText>>,
    chalk: Res<LocalChalk>,
    stats: Res<Stats>,
) {
    let mut text = text.single_mut();
    let chalk = chalk.get();

    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::smoothed)
        .map_or("-- fps".to_owned(), |x| format!("{x:.1} fps"));

    let frame_time = {
        let t = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(Diagnostic::smoothed)
            .unwrap_or_else(|| time.delta_seconds_f64());
        format!("{t:.3} ms/frame")
    };

    let chalk = {
        let x = chalk.x;
        let y = chalk.y;
        let pressed = chalk.pressed;
        format!("{x:+05}:{y:+05} {pressed}")
    };

    let collab = { format!("{:?}", stats.collab) };

    let text_value = &mut text.sections[0].value;

    *text_value = format!(
        r#"{fps}
{frame_time}
{chalk}
{collab}
"#
    );
}
