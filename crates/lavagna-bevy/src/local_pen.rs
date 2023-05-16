use crate::MainCamera;
use crate::Chalk;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::Window,
};

pub(crate) struct LocalPenPlugin;

#[derive(Component)]
struct LocalPen;

impl Plugin for LocalPenPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup)
            .add_system(handle_user_input)
            .add_system(update_sprite_position);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((LocalPen, Chalk::new()));

    commands.spawn((
        LocalPen,
        SpriteBundle {
            texture: asset_server.load("sprites/pen.png"),
            transform: Transform {
                scale: Vec3::new(0.2, 0.2, 0.2),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

fn handle_user_input(
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut pen_q: Query<&mut Chalk, With<LocalPen>>,
) {
    let (camera, camera_transform) = camera_q.single();
    let mut pen = pen_q.single_mut();
    let window = window_q.single();

    let prev_pen = *pen;

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

    pen.updated = is_updated(&prev_pen, &pen);
}

fn update_sprite_position(
    pen_q: Query<&mut Chalk, With<LocalPen>>,
    mut sprite_transform_q: Query<&mut Transform, With<LocalPen>>,
    mut sprite_visibility_q: Query<&mut Visibility, With<LocalPen>>,
) {
    let pen = pen_q.single();
    let mut t = sprite_transform_q.single_mut();
    let mut v = sprite_visibility_q.single_mut();
    t.translation = Vec3::new(pen.x as f32, pen.y as f32, 0.);

    *v = if pen.pressed {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn is_updated(old_pen: &Chalk, new_pen: &Chalk) -> bool {
    old_pen.x != new_pen.x || old_pen.y != new_pen.y
}
