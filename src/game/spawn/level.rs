//! Spawn the main level by triggering other observers.

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use super::player::SpawnPaddle;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shapes = [
        Mesh2dHandle(meshes.add(Circle { radius: 30.0 })),
        Mesh2dHandle(meshes.add(Annulus::new(280.0, 300.0))),
    ];
    let num_shapes = shapes.len();
    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn(MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
    }

    commands.trigger(SpawnPaddle);
}
