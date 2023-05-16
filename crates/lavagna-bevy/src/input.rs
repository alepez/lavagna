use crate::LocalPen;
use crate::MainCamera;
use crate::Pen;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::{Camera, EventReader, GlobalTransform, MouseButton, Query, With},
    window::Window,
};

pub(crate) fn input_system(
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut pen_q: Query<&mut Pen, With<LocalPen>>,
) {
    let (camera, camera_transform) = camera_q.single();
    let mut pen = pen_q.single_mut();
    let window = window_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        pen.x = world_position[0] as i64;
        pen.y = world_position[1] as i64;
    }

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
}
