use crate::Chalk;
use crate::MainCamera;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::Window,
};
use bevy_prototype_lyon::prelude::Fill;
use bevy_prototype_lyon::prelude::GeometryBuilder;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_prototype_lyon::shapes;

pub(crate) struct LocalPenPlugin;

#[derive(Resource)]
pub(crate) struct LocalChalkConfig {
    pub(crate) color: Color,
    pub(crate) line_width: u32,
}

impl Default for LocalChalkConfig {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            line_width: 8,
        }
    }
}

#[derive(Component)]
pub struct LocalChalk;

impl Plugin for LocalPenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalChalkConfig>()
            .add_startup_system(startup)
            .add_system(handle_user_input)
            .add_system(update_position)
            .add_system(update_config);
    }
}

fn startup(mut commands: Commands, chalk_config: Res<LocalChalkConfig>) {
    commands.spawn((LocalChalk, Chalk::new()));

    let shape = shapes::Circle {
        radius: chalk_config.line_width as f32 / 2.,
        center: Vec2::new(0.0, 0.0),
    };

    commands.spawn((
        LocalChalk,
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Fill::color(chalk_config.color),
    ));
}

fn handle_user_input(
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut pen_q: Query<&mut Chalk, With<LocalChalk>>,
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

fn update_position(
    pen_q: Query<&mut Chalk, With<LocalChalk>>,
    mut sprite_transform_q: Query<&mut Transform, With<LocalChalk>>,
) {
    let pen = pen_q.single();
    let mut t = sprite_transform_q.single_mut();
    t.translation = Vec3::new(pen.x as f32, pen.y as f32, 0.);
}

fn is_updated(old_pen: &Chalk, new_pen: &Chalk) -> bool {
    old_pen.x != new_pen.x || old_pen.y != new_pen.y
}

fn update_config(
    chalk_config: Res<LocalChalkConfig>,
    mut chalk_q: Query<&mut Chalk, With<LocalChalk>>,
    mut fill_q: Query<&mut Fill, With<LocalChalk>>,
) {
    let chalk = &mut chalk_q.single_mut();
    chalk.color = chalk_config.color;
    chalk.line_width = chalk_config.line_width;

    let fill: &mut Fill = &mut fill_q.single_mut();
    *fill = Fill::color(chalk_config.color);
}
