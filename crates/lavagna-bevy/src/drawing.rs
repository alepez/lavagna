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
    mut commands: Commands,
    pens_q: Query<&Pen>,
    mut polyline_q: Query<&mut Polyline>,
    mut path_q: Query<&mut Path, With<Pending>>,
) {
    let pen = pens_q.single();
    let polyline: &mut Polyline = &mut polyline_q.single_mut();

    let update = pen.pressed && pen.updated;
    let just_released = !pen.pressed && !polyline.points.is_empty();

    if update {
        let new_point = Vec2::new(pen.x as f32, pen.y as f32);
        polyline.points.push(new_point);

        *path_q.single_mut() = Path::from(&*polyline);
    }

    if just_released {
        commands.spawn((
            ShapeBundle {
                path: Path::from(&*polyline),
                ..default()
            },
            Stroke::new(Color::WHITE, 10.0),
            Fill::color(Color::NONE),
            Completed,
        ));

        polyline.points.clear();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Polyline::default());

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
struct Polyline {
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
