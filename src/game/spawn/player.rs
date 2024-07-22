use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_paddle);
}

#[derive(Event, Debug)]
pub struct SpawnPaddle;

#[derive(Component, Debug)]
pub struct Paddle;

#[derive(Component, Debug)]
pub struct PaddleRotation;

fn spawn_paddle(
    _trigger: Trigger<SpawnPaddle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((SpatialBundle::default(), PaddleRotation))
        .with_children(|b| {
            b.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Capsule2d::new(25.0, 100.0))),
                    material: materials.add(Color::BLACK),
                    transform: Transform::from_xyz(285.0, 0.0, 1.0),
                    ..default()
                },
                Paddle,
            ));
        });
}
