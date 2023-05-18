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

#[derive(Component)]
struct LocalCursor;

#[derive(Resource)]
pub struct LocalChalk(Chalk);

impl LocalChalk {
    pub(crate) fn get_mut(&mut self) -> &mut Chalk {
        &mut self.0
    }

    pub(crate) fn get(&self) -> &Chalk {
        &self.0
    }
}

impl Default for LocalChalk {
    fn default() -> Self {
        Self(Chalk {
            color: Color::WHITE,
            line_width: 8,
            pressed: false,
            updated: false,
            x: 0,
            y: 0,
        })
    }
}

impl Plugin for LocalPenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalChalk>()
            .add_startup_system(startup)
            .add_system(handle_user_input)
            .add_system(update_position)
            .add_system(update_pressed)
            .add_system(update_config);
    }
}

fn startup(mut commands: Commands, chalk: Res<LocalChalk>) {
    let chalk = &chalk.0;

    let shape = shapes::Circle {
        radius: 1.0,
        center: Vec2::new(0.0, 0.0),
    };

    commands.spawn((
        LocalCursor,
        Chalk::new(),
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Fill::color(chalk.color),
    ));
}

fn handle_user_input(
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut chalk: ResMut<LocalChalk>,
) {
    let (camera, camera_transform) = camera_q.single();
    let chalk = &mut chalk.0;
    let window = window_q.single();

    let prev_chalk = *chalk;

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        chalk.x = world_position[0] as i32;
        chalk.y = world_position[1] as i32;
    }

    chalk.updated = is_updated(&prev_chalk, &chalk);
}

fn update_pressed(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut chalk: ResMut<LocalChalk>,
) {
    let chalk = &mut chalk.0;

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
    mut chalk: ResMut<LocalChalk>,
    mut cursor_q: Query<&mut Transform, With<LocalCursor>>,
) {
    let chalk = &mut chalk.0;
    let mut t = cursor_q.single_mut();
    t.translation = Vec3::new(chalk.x as f32, chalk.y as f32, 0.);
}

fn is_updated(old_chalk: &Chalk, new_chalk: &Chalk) -> bool {
    old_chalk.x != new_chalk.x || old_chalk.y != new_chalk.y
}

fn update_config(
    mut chalk: ResMut<LocalChalk>,
    mut cursor_q: Query<(&mut Fill, &mut Transform), With<LocalCursor>>,
) {
    let chalk = &mut chalk.0;
    let (mut fill, mut transform) = cursor_q.single_mut();

    // chalk.color = chalk_config.color;
    // chalk.line_width = chalk_config.line_width;

    *fill = Fill::color(chalk.color);

    let scale = chalk.line_width as f32 / 2.0;
    transform.scale = Vec3::new(scale, scale, scale);
}
