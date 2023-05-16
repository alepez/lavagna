use crate::Pen;
use bevy::prelude::*;

use bevy::render::mesh::PrimitiveTopology;
use bevy::sprite::MaterialMesh2dBundle;

pub(crate) struct DrawingPlugin;

impl Plugin for DrawingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup).add_system(update);
    }
}

fn update(pens_q: Query<&Pen>) {
    for pen in &pens_q {
        if pen.pressed {
            dbg!(&pen);
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(MaterialMesh2dBundle {
        mesh: bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(LineList {
            lines: vec![
                (Vec3::ZERO, Vec3::new(100.0, 100.0, 0.0)),
                (Vec3::new(100.0, 100.0, 0.0), Vec3::new(100.0, 0.0, 0.0)),
            ],
        }))),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        ..default()
    });
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}
