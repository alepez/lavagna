#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

mod collab;
mod debug;
mod drawing;
mod keybinding;
mod local_chalk;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

#[cfg(target_arch = "wasm32")]
pub mod web;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::{prelude::*, window::Window};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::collab::CollabPlugin;
pub use crate::collab::CollabPluginOpt as CollabOpt;
use crate::debug::DebugPlugin;
use crate::drawing::DrawingPlugin;
use crate::keybinding::KeybindingPlugin;
use crate::local_chalk::LocalChalkPlugin;
use crate::ui::UiPlugin;

#[derive(Debug, Default)]
pub struct Opt {
    pub collab: Option<CollabOpt>,
    pub show_debug_pane: bool,
    pub verbose: bool,
    pub ui: bool,
}

pub fn run(opt: Opt) {
    let mut app = App::new();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            // Avoid scrolling on mobile
            prevent_default_event_handling: true,
            canvas: Some("#bevy".to_owned()),
            ..default()
        }),
        ..default()
    };

    let log_plugin = if opt.verbose {
        LogPlugin {
            filter: "warn,lavagna=debug,wgpu_core=warn,wgpu_hal=warn".into(),
            level: bevy::log::Level::DEBUG,
        }
    } else {
        LogPlugin {
            filter: default(),
            level: bevy::log::Level::ERROR,
        }
    };

    app.add_plugins(
        DefaultPlugins
            .set(window_plugin)
            .set(log_plugin)
            .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin::default()),
    );

    app.insert_resource(Stats::default());

    app.add_systems(Startup, setup);

    app.add_plugins(KeybindingPlugin);
    app.add_plugins(FramepacePlugin);
    app.add_plugins(LocalChalkPlugin);
    app.add_plugins(DrawingPlugin);
    app.add_plugins(PanCamPlugin);

    if opt.show_debug_pane {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_plugins(DebugPlugin);
    }

    if let Some(collab_opt) = opt.collab {
        app.add_plugins(CollabPlugin::new(collab_opt));
    }

    app.add_plugins(UiPlugin::new(opt.ui));

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

#[must_use]
pub fn options() -> Opt {
    #[cfg(target_arch = "wasm32")]
    {
        web::options_from_url()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        cli::options_from_args()
    }
}

#[derive(Debug, Default)]
pub struct CollabStats {
    pub active: bool,
    pub peers: usize,
}

#[derive(Debug, Resource, Default)]
pub struct Stats {
    pub collab: CollabStats,
}
