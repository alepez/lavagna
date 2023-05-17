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
            .add_system(update_pressed)
            .add_system(update_config);
    }
}

fn startup(mut commands: Commands, chalk_config: Res<LocalChalkConfig>) {
    let shape = shapes::Circle {
        radius: 1.0,
        center: Vec2::new(0.0, 0.0),
    };

    commands.spawn((
        LocalChalk,
        Chalk::new(),
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
    mut chalk_q: Query<&mut Chalk, With<LocalChalk>>,
) {
    let (camera, camera_transform) = camera_q.single();
    let mut chalk = chalk_q.single_mut();
    let window = window_q.single();

    let prev_chalk = *chalk;

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        chalk.x = world_position[0] as i64;
        chalk.y = world_position[1] as i64;
    }

    chalk.updated = is_updated(&prev_chalk, &chalk);
}

fn is_ctrl_pressed(keyboard_input: &Input<KeyCode>) -> bool {
    keyboard_input.pressed(KeyCode::LControl) || keyboard_input.pressed(KeyCode::RControl)
}

fn update_pressed(
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut chalk_q: Query<&mut Chalk, With<LocalChalk>>,
) {
    let mut chalk = chalk_q.single_mut();

    if is_ctrl_pressed(&keyboard_input) {
        return;
    }

    for event in mouse_button_input_events.iter() {
        match event {
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            } => {
                chalk.pressed = true;
            }
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
            } => {
                chalk.pressed = false;
            }
            _ => {}
        }
    }
}

fn update_position(
    chalk_q: Query<&mut Chalk, With<LocalChalk>>,
    mut sprite_transform_q: Query<&mut Transform, With<LocalChalk>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if is_ctrl_pressed(&keyboard_input) {
        return;
    }

    let chalk = chalk_q.single();
    let mut t = sprite_transform_q.single_mut();
    t.translation = Vec3::new(chalk.x as f32, chalk.y as f32, 0.);
}

fn is_updated(old_chalk: &Chalk, new_chalk: &Chalk) -> bool {
    old_chalk.x != new_chalk.x || old_chalk.y != new_chalk.y
}

fn update_config(
    chalk_config: Res<LocalChalkConfig>,
    mut chalk_q: Query<(&mut Chalk, &mut Fill, &mut Transform), With<LocalChalk>>,
) {
    let (mut chalk, mut fill, mut transform) = chalk_q.single_mut();

    chalk.color = chalk_config.color;
    chalk.line_width = chalk_config.line_width;

    *fill = Fill::color(chalk_config.color);

    let scale = chalk.line_width as f32 / 2.0;
    transform.scale = Vec3::new(scale, scale, scale);
}
