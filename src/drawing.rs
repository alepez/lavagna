use crate::Chalk;
use bevy::prelude::*;

use bevy_prototype_lyon::prelude::*;

pub(crate) struct DrawingPlugin;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ShapePlugin)
            .add_event::<ClearEvent>()
            .add_system(handle_clear_event)
            .add_system(update);
    }
}

fn update(
    mut commands: Commands,
    mut chalk_q: Query<(&mut Chalk, &mut Path, &mut Stroke, &mut Polyline), With<Pending>>,
    time: Res<Time>,
) {
    for (chalk, mut path, mut stroke, mut polyline) in chalk_q.iter_mut() {
        let update = chalk.pressed && chalk.updated;
        let just_released = !chalk.pressed && !polyline.points.is_empty();

        stroke.color = chalk.color;
        stroke.options.line_width = chalk.line_width as f32;

        if just_released {
            complete_pending_path(&mut polyline, &mut commands, &chalk, &time);
        } else if update {
            add_point(&mut polyline, &chalk);
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
    let t = time.elapsed_seconds();
    const MAX_Z: f32 = 500.0;
    const MAX_TIME: f32 = 10_000.0;
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

    // z-index at maximum before clipping pane
    let transform = Transform {
        translation: Vec3::new(0., 0., 999.0),
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
        despawn_all_completed_lines(&mut commands, &lines)
    }
}
