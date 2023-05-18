mod collab;
mod debug;
mod drawing;
mod keybinding;
mod local_chalk;

use bevy::{prelude::*, window::Window};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::collab::CollabPlugin;
pub use crate::collab::CollabPluginOpt as CollabOpt;
use crate::debug::DebugPlugin;
use crate::drawing::DrawingPlugin;
use crate::keybinding::KeybindingPlugin;
use crate::local_chalk::LocalPenPlugin;

#[derive(Debug)]
pub struct Opt {
    pub collab: Option<CollabOpt>,
}

pub fn run(opt: Opt) {
    dbg!(&opt);

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (640., 480.).into(),
                    ..default()
                }),
                ..default()
            })
            .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
    );

    app.add_plugin(FramepacePlugin);
    app.add_plugin(DebugPlugin);
    app.add_plugin(LocalPenPlugin);
    app.add_plugin(DrawingPlugin);
    app.add_plugin(KeybindingPlugin);
    app.add_plugin(PanCamPlugin::default());
    app.add_startup_system(setup);

    if let Some(collab_opt) = opt.collab {
        app.add_plugin(CollabPlugin::new(collab_opt));
    }

    app.run();
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

#[derive(Component, Debug, Clone, Copy, Default)]
struct Chalk {
    pressed: bool,
    updated: bool,
    just_released: bool,
    x: i32,
    y: i32,
    color: Color,
    line_width: u32,
}
