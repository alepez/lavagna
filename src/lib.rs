mod debug;
mod drawing;
mod keybinding;
mod local_chalk;
mod collab;

use bevy::{prelude::*, window::Window};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::debug::DebugPlugin;
use crate::drawing::DrawingPlugin;
use crate::keybinding::KeybindingPlugin;
use crate::local_chalk::LocalPenPlugin;
use crate::collab::CollabPlugin;

pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (640., 480.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(FramepacePlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(LocalPenPlugin)
        .add_plugin(DrawingPlugin)
        .add_plugin(KeybindingPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(CollabPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut clear_color: ResMut<ClearColor>,
    mut framepace: ResMut<FramepaceSettings>,
) {
    commands
        .spawn((Camera2dBundle::default(), MainCamera))
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.1,
            max_scale: Some(10.0),
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
        });

    clear_color.0 = Color::BLACK;

    // Limit frame rate, we dont't want to squeeze that CPU
    framepace.limiter = Limiter::from_framerate(30.0);
}

#[derive(Component, Debug, Clone, Copy)]
struct Chalk {
    pressed: bool,
    updated: bool,
    x: i32,
    y: i32,
    color: Color,
    line_width: u32,
}

impl Chalk {
    fn new() -> Self {
        Self {
            pressed: false,
            updated: false,
            x: 0,
            y: 0,
            color: Color::WHITE,
            line_width: 10,
        }
    }
}