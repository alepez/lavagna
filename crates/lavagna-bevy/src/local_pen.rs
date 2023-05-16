use crate::MainCamera;
use crate::Pen;

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
    commands.spawn((LocalPen, Pen::new()));

    commands.spawn((
        LocalPen,
        SpriteBundle {
            texture: asset_server.load("sprites/pen.png"),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

fn handle_user_input(
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

fn update_sprite_position(
    pen_q: Query<&mut Pen, With<LocalPen>>,
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
