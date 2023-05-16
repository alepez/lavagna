use crate::{local_chalk::LocalChalk, Chalk};
use bevy::prelude::*;

use bevy_prototype_lyon::prelude::*;

pub(crate) struct DrawingPlugin;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ShapePlugin)
            .add_startup_system(setup)
            .add_system(update);
    }
}

fn update(
    commands: Commands,
    chalk_q: Query<&Chalk, With<LocalChalk>>,
    mut polyline_q: Query<&mut PendingPolyline>,
    mut path_q: Query<&mut Path, With<Pending>>,
    mut stroke_q: Query<&mut Stroke, With<Pending>>,
) {
    let chalk = chalk_q.single();
    let polyline: &mut PendingPolyline = &mut polyline_q.single_mut();
    let stroke = &mut stroke_q.single_mut();

    let update = chalk.pressed && chalk.updated;
    let just_released = !chalk.pressed && !polyline.points.is_empty();

    stroke.color = chalk.color;
    stroke.options.line_width = chalk.line_width as f32;

    if just_released {
        complete_pending_path(polyline, commands, chalk);
    } else if update {
        add_point(polyline, chalk);
    }

    // Regenerate mesh from list of points
    *path_q.single_mut() = Path::from(&*polyline);
}

fn add_point(polyline: &mut PendingPolyline, chalk: &Chalk) {
    let new_point = Vec2::new(chalk.x as f32, chalk.y as f32);
    polyline.points.push(new_point);
}

fn complete_pending_path(polyline: &mut PendingPolyline, mut commands: Commands, chalk: &Chalk) {
    let path = Path::from(&*polyline);

    commands.spawn((
        ShapeBundle { path, ..default() },
        Stroke::new(chalk.color, chalk.line_width as f32),
        Fill::color(Color::NONE),
        Completed,
    ));

    polyline.points.clear();
}

fn setup(mut commands: Commands) {
    commands.spawn(PendingPolyline::default());

    let path_builder = PathBuilder::new();
    let path = path_builder.build();

    commands.spawn((
        ShapeBundle { path, ..default() },
        Stroke::new(Color::WHITE, 10.0),
        Fill::color(Color::NONE),
        Pending,
    ));
}

#[derive(Debug, Component, Default)]
struct Completed;

#[derive(Debug, Component, Default)]
struct Pending;

#[derive(Debug, Clone, Component, Default)]
struct PendingPolyline {
    points: Vec<Vec2>,
}

impl From<&PendingPolyline> for Path {
    fn from(polyline: &PendingPolyline) -> Self {
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
