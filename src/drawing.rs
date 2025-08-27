#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_precision_loss)]

use crate::Chalk;
use bevy::{math::NormedVectorSpace, prelude::*};

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
        let updated = chalk.pressed
            && chalk.updated
            && polyline.points.last() != Some(&Vec2::new(chalk.x as f32, chalk.y as f32));

        stroke.color = chalk.color.into();
        stroke.options.line_width = chalk.line_width as f32;

        if updated {
            if let Some(last_points) = polyline.points.last_chunk::<2>().copied() {
                // Check whether adding the next point will cause Lyon to triangulate the path incorrectly.
                // If it will, complete the path and start a new path to avoid the bug.
                if triggers_lyon_bug(
                    &[
                        last_points[0],
                        last_points[1],
                        Vec2::new(chalk.x as f32, chalk.y as f32),
                    ],
                    chalk.line_width as f32 / 2.0, // Circle radius is half the line width
                ) {
                    complete_pending_path(&mut polyline, &mut commands, &chalk, &time);
                    polyline.points.push(last_points[1]);
                }
            }

            // Extend the path normally.
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
            spatial: transform.into(),
            ..default()
        },
        Stroke {
            color: chalk.color.into(),
            options: StrokeOptions::default()
                .with_line_width(chalk.line_width as f32)
                .with_line_join(LineJoin::Round)
                .with_line_cap(LineCap::Round),
        },
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
            spatial: transform.into(),
            ..default()
        },
        Stroke {
            color: Color::WHITE,
            options: StrokeOptions::default()
                .with_line_width(10.0)
                .with_line_join(LineJoin::Round)
                .with_line_cap(LineCap::Round),
        },
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
    let clear = events.read().count() > 0;
    if clear {
        despawn_all_completed_lines(&mut commands, &lines);
    }
}

/// Returns whether Lyon would triangulate the given 3-vertex path incorrectly.
/// This path should not consist of any segments of length 0 (so adjacent points should not be repeated).
fn triggers_lyon_bug(points: &[Vec2; 3], stroke_radius: f32) -> bool {
    // The issue at hand is https://github.com/nical/lyon/issues/891.
    // Lyon expects the stroke radius to be small and for each segment of the path to be long.
    // When this is the case, the inner corner between two path segments can be computed as
    // the intersection between two lines (the ones offset by `stroke_radius` from the line segments forming the path).
    // However, this assumption does not hold when the stroke radius is too large compared to the line segment lengths.

    // Here is the math to determine when this is a problem:
    // Lyon expects the inner corner to be `stroke_radius` units away from both line segments, so if we
    // drop a perpendicular from the expected inner corner location to each of the two line segments,
    // the length of the perpendicular will be `stroke_radius`, and the distance from the landing point
    // of the perpendicular to the origin will be the cutoff in which line segments shorter than that
    // will cause buggy behavior in Lyon.

    // If we draw a diagram illustrating the above as well as connecting the inner corner to `points[1]` with a line
    // segment, two congruent triangles will appear, bisecting the angle between the two main line segments.
    // This produces the following formula:
    // `tan(theta / 2) == stroke_radius / min_length`
    // which is equivalent to
    // `min_length * sin(theta / 2) == cos(theta / 2) * stroke_radius`
    // which is equivalent to (because both sides are positive)
    // `min_length^2 * sin^2(theta / 2) == cos^2(theta / 2) * stroke_radius^2`

    // The following code uses the above formula along with other trigonometric identities, being careful
    // to avoid any possible division by 0 unless one of the path segments has length 0 (forbidden as a precondition).
    // Square roots are also avoided in case floating point precision would result in the square root of a negative number.
    let vector0 = points[0] - points[1];
    let vector1 = points[2] - points[1];
    let cos_angle = vector0.dot(vector1) / (vector0.norm() * vector1.norm());

    let sqr_cos_half_angle = 1.0 + cos_angle;
    let sqr_sin_half_angle = 1.0 - cos_angle;

    let sqr_stroke_radius = stroke_radius * stroke_radius;

    vector0.norm_squared() * sqr_sin_half_angle <= sqr_cos_half_angle * sqr_stroke_radius
        || vector1.norm_squared() * sqr_sin_half_angle <= sqr_cos_half_angle * sqr_stroke_radius
}
