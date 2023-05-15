use bevy::{input::mouse::*, prelude::*};

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
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn mouse_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    for event in mouse_button_input_events.iter() {
        info!("{:?}", event);
    }

    for event in cursor_moved_events.iter() {
        info!("{:?}", event);
    }
}
