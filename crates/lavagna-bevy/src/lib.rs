mod debug;
mod local_pen;
mod drawing;

use bevy::{prelude::*, window::Window};

use crate::debug::DebugPlugin;
use crate::local_pen::LocalPenPlugin;
use crate::drawing::DrawingPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (640., 480.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(DebugPlugin)
        .add_plugin(LocalPenPlugin)
        .add_plugin(DrawingPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

#[derive(Component, Debug)]
struct Pen {
    pressed: bool,
    x: i64,
    y: i64,
}

impl Pen {
    fn new() -> Self {
        Self {
            pressed: false,
            x: 0,
            y: 0,
        }
    }
}
