use bevy::{
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
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(Pen::new_local());
}

fn mouse_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut query: Query<&mut Pen>,
) {
    for mut pen in &mut query {
        if pen.local {
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
    }
}

#[derive(Component)]
struct Pen {
    pressed: bool,
    local: bool,
    x: i64,
    y: i64,
}

impl Pen {
    fn new_local() -> Self {
        Self {
            pressed: true,
            local: true,
            x: 0,
            y: 0,
        }
    }
}
