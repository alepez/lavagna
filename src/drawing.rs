#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_precision_loss)]

use crate::Chalk;
use bevy::prelude::*;

use bevy_prototype_lyon::prelude::*;

const POINTS_CHUNK_THRESHOLD: usize = 100;

pub(crate) struct DrawingPlugin;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(ShapePlugin)
            .add_event::<ClearEvent>()
            .add_systems(Update, handle_clear_event)
            .add_systems(Update, update);
    }
}

fn update(
    mut commands: Commands,
    mut chalk_q: Query<(&mut Chalk, &mut Path, &mut Stroke, &mut Polyline), With<Pending>>,
    time: Res<Time>,
) {
    for (chalk, mut path, mut stroke, mut polyline) in &mut chalk_q {
        let updated = chalk.pressed && chalk.updated;

        stroke.color = chalk.color;
        stroke.options.line_width = chalk.line_width as f32;

        if updated {
            add_point(&mut polyline, &chalk);
        }

        let chunk_completed = polyline.points.len() >= POINTS_CHUNK_THRESHOLD;
        let just_released = chalk.just_released && !polyline.points.is_empty();
        let completed = just_released || chunk_completed;

        if completed {
            complete_pending_path(&mut polyline, &mut commands, &chalk, &time);

            if chunk_completed {
                add_point(&mut polyline, &chalk);
            }
        }

        // Regenerate mesh from list of points
        *path = Path::from(&*polyline);
    }
}

fn add_point(polyline: &mut Polyline, chalk: &Chalk) {
    let new_point = Vec2::new(chalk.x as f32, chalk.y as f32);
    polyline.points.push(new_point);
}

fn z_from_time(time: &Time) -> f32 {
    const MAX_Z: f32 = 500.0;
    const MAX_TIME: f32 = 10_000.0;

    let t = time.elapsed_seconds();
    let step = MAX_Z / MAX_TIME;
    t * step
}

fn complete_pending_path(
    polyline: &mut Polyline,
    commands: &mut Commands,
    chalk: &Chalk,
    time: &Time,
) {
    let path = Path::from(&*polyline);

    let z = z_from_time(time);

    let transform = Transform {
        translation: Vec3::new(0., 0., z),
        ..default()
    };

    // The line is complete, we spawn a fresh mesh, which will persist
    commands.spawn((
        ShapeBundle {
            path,
            transform,
            ..default()
        },
        Stroke::new(chalk.color, chalk.line_width as f32),
        Fill::color(Color::NONE),
        Completed,
    ));

    polyline.points.clear();
}

pub(crate) fn make_chalk(chalk: Chalk) -> (ShapeBundle, Stroke, Fill, Polyline, Pending, Chalk) {
    // An empty path
    let path = PathBuilder::new().build();

    // z-index at maximum before cursor plane
    let transform = Transform {
        translation: Vec3::new(0., 0., 998.0),
        ..default()
    };

    (
        ShapeBundle {
            path,
            transform,
            ..default()
        },
        Stroke::new(Color::WHITE, 10.0),
        Fill::color(Color::NONE),
        Polyline::default(),
        Pending,
        chalk,
    )
}

#[derive(Debug, Component, Default)]
struct Completed;

#[derive(Debug, Component, Default)]
pub(crate) struct Pending;

#[derive(Debug, Clone, Component, Default)]
pub(crate) struct Polyline {
    points: Vec<Vec2>,
}

impl From<&Polyline> for Path {
    fn from(polyline: &Polyline) -> Self {
        let mut path_builder = PathBuilder::new();

        let mut iter = polyline.points.iter();

        if let Some(first) = iter.next() {
            path_builder.move_to(*first);
        }

        for point in iter {
            path_builder.line_to(*point);
        }

        path_builder.build()
    }
}

fn despawn_all_completed_lines(commands: &mut Commands, lines: &Query<Entity, With<Completed>>) {
    for line in lines.iter() {
        commands.entity(line).despawn();
    }
}

#[derive(Event)]
pub(crate) struct ClearEvent(bool);

impl ClearEvent {
    pub(crate) fn new() -> Self {
        Self(true)
    }
    pub(crate) fn local_only() -> Self {
        Self(false)
    }
    pub(crate) fn must_be_forwarded(&self) -> bool {
        self.0
    }
}

fn handle_clear_event(
    mut events: EventReader<ClearEvent>,
    lines: Query<Entity, With<Completed>>,
    mut commands: Commands,
) {
    let clear = events.iter().count() > 0;
    if clear {
        despawn_all_completed_lines(&mut commands, &lines);
    }
}
