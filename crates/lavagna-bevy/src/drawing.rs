use crate::Pen;
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
    pens_q: Query<&Pen>,
    mut polyline_q: Query<&mut PendingPolyline>,
    mut path_q: Query<&mut Path, With<Pending>>,
) {
    let pen = pens_q.single();
    let polyline: &mut PendingPolyline = &mut polyline_q.single_mut();

    let update = pen.pressed && pen.updated;
    let just_released = !pen.pressed && !polyline.points.is_empty();

    if just_released {
        complete_pending_path(polyline, commands);
    } else if update {
        add_point(polyline, pen);
        *path_q.single_mut() = Path::from(&*polyline);
    }
}

fn add_point(polyline: &mut PendingPolyline, pen: &Pen) {
    let new_point = Vec2::new(pen.x as f32, pen.y as f32);
    polyline.points.push(new_point);
}

fn complete_pending_path(polyline: &mut PendingPolyline, mut commands: Commands) {
    let path = Path::from(&*polyline);

    commands.spawn((
        ShapeBundle { path, ..default() },
        Stroke::new(Color::WHITE, 10.0),
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
